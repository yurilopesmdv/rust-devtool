#!/usr/bin/env python3
"""Validate native Mongo migration executors against an explicitly disposable standalone."""

import argparse
import concurrent.futures
import json
import os
import signal
import subprocess
import tempfile
import time
from pathlib import Path

if __package__:
    from .lock_support import run
    from .process_support import capture
else:
    from lock_support import run
    from process_support import capture

STACKS = ["express-ts", "nest-ts", "python-fastapi", "rust-axum"]
ADMIN = """
import {MongoClient} from 'mongodb';
const client=new MongoClient(process.env.MONGODB_TEST_URL,{serverSelectionTimeoutMS:5000,socketTimeoutMS:5000});
try {
 await client.connect();const db=client.db();const ledger=db.collection('_boilerplate_migrations');const locks=db.collection('_boilerplate_migration_lock');let result;
 switch(process.env.MIGRATION_TEST_ACTION){
 case 'empty':result=(await db.listCollections().toArray()).length===0;break;
 case 'setup':await db.collection('examples').insertMany([{_id:'a_valid',message:'primeiro'},{_id:'b_invalid',message:123}]);result=true;break;
 case 'failed':{const row=await ledger.findOne({_id:'20261027000200'});result={status:row?.status,checkpoint:Number(row?.checkpoint??0),copied:(await db.collection('examples').findOne({_id:'a_valid'}))?.display_name};break;}
 case 'fix':await db.collection('examples').updateOne({_id:'b_invalid'},{$set:{message:'corrigido'}});result=true;break;
 case 'complete':result={count:await db.collection('examples').countDocuments({display_name:{$exists:true}}),rows:await ledger.find().project({_id:1,status:1,checkpoint:1}).toArray(),lease:await locks.findOne({_id:'runner'})};break;
 case 'reserve':await locks.insertOne({_id:'runner',owner:'test-stale',started_at:new Date()});result=true;break;
 case 'reserved':result=(await locks.findOne({_id:'runner'}))?.owner??null;break;
 case 'drop':for(const name of ['examples','_boilerplate_migrations','_boilerplate_migration_lock'])if(await db.listCollections({name}).hasNext())await db.collection(name).drop();result=true;break;
 default:throw new Error('unknown action');
 }
 console.log(JSON.stringify(result));
}finally{await client.close();}
"""


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--disposable-database", action="store_true", required=True)
    parser.add_argument("--stacks", nargs="+", choices=STACKS, default=STACKS)
    parser.add_argument("--offline", action="store_true")
    args = parser.parse_args()
    env = os.environ.copy()
    if not env.get("MONGODB_TEST_URL"):
        parser.error("Defina MONGODB_TEST_URL de um standalone vazio descartável")
    if args.offline:
        env.update(
            CARGO_NET_OFFLINE="true", npm_config_offline="true", UV_OFFLINE="true"
        )
    for key in ["DATABASE_URL", "DATABASE_URL_FILE", "MIGRATION_DATABASE_URL_FILE"]:
        env.pop(key, None)
    env["MIGRATION_DATABASE_URL"] = env["MONGODB_TEST_URL"]
    repo = Path(__file__).resolve().parents[3]
    run(["cargo", "build", "--locked", "-p", "rust-boilerplate"], repo, env)
    metadata = json.loads(
        run(["cargo", "metadata", "--no-deps", "--format-version=1"], repo, env)
    )
    binary = str(Path(metadata["target_directory"]) / "debug/boilerplate")
    with tempfile.TemporaryDirectory(prefix="native mongo migrations-") as temporary:
        root = Path(temporary)
        control = root / "control"
        run(
            [
                binary,
                "new",
                "mongo-control",
                "--type",
                "backend",
                "--backend",
                "express-ts",
                "--database",
                "mongodb",
                "--database-location",
                "external",
                "--destination",
                str(control),
                "--install",
                "no",
                "--git",
                "no",
                "--github",
                "no",
                "--non-interactive",
            ],
            root,
            env,
        )
        run(["npm", "ci", "--no-audit", "--no-fund"], control, env)

        def admin(action):
            control_env = env.copy()
            control_env["MIGRATION_TEST_ACTION"] = action
            return json.loads(
                run(["node", "--input-type=module", "-e", ADMIN], control, control_env)
            )

        if not admin("empty"):
            parser.error("Banco não vazio; validação recusada sem alterar dados")
        for stack in args.stacks:
            app = root / stack
            run(
                [
                    binary,
                    "new",
                    "native-migrations",
                    "--type",
                    "backend",
                    "--backend",
                    stack,
                    "--database",
                    "mongodb",
                    "--database-location",
                    "external",
                    "--migrations",
                    "mongodb-native",
                    "--destination",
                    str(app),
                    "--install",
                    "no",
                    "--git",
                    "no",
                    "--github",
                    "no",
                    "--non-interactive",
                ],
                root,
                env,
            )
            app_env = env.copy()
            if stack in ["express-ts", "nest-ts"]:
                run(["npm", "ci", "--no-audit", "--no-fund"], app, app_env)
                for script in ["lint", "format:check", "typecheck", "build"]:
                    run(["npm", "run", script], app, app_env)
                command = ["node", "scripts/migrate.mjs"]
            elif stack == "python-fastapi":
                run(["uv", "sync", "--locked"], app, app_env)
                run(["uv", "run", "--locked", "ruff", "check", "."], app, app_env)
                run(
                    ["uv", "run", "--locked", "ruff", "format", "--check", "."],
                    app,
                    app_env,
                )
                command = ["uv", "run", "--locked", "python", "scripts/migrate.py"]
            else:
                app_env.setdefault("CARGO_TARGET_DIR", str(root / "rust-target"))
                run(["cargo", "fmt", "--check"], app, app_env)
                run(["cargo", "build", "--locked", "--bin", "migrate"], app, app_env)
                run(
                    [
                        "cargo",
                        "clippy",
                        "--locked",
                        "--all-targets",
                        "--",
                        "-D",
                        "warnings",
                    ],
                    app,
                    app_env,
                )
                command = [str(Path(app_env["CARGO_TARGET_DIR"]) / "debug/migrate")]

            def migrate(
                arguments=None, success=True, app=app, command=command, app_env=app_env
            ):
                result = capture(
                    [*command, *(arguments or ["up"])], app, app_env, timeout=120
                )
                if (result.returncode == 0) != success:
                    raise RuntimeError(
                        f"Executor nativo falhou em {app.name}; código {result.returncode}"
                    )
                if env["MONGODB_TEST_URL"] in result.stdout + result.stderr:
                    raise RuntimeError("URI exposta no diagnóstico")
                return json.loads(result.stdout) if success else None

            try:
                assert migrate(["status"])["migrations"][0]["status"] == "pending"
                secret = root / "migration-url"
                secret.write_text(env["MONGODB_TEST_URL"])
                secret.chmod(0o600)
                file_env = app_env.copy()
                file_env.pop("MIGRATION_DATABASE_URL", None)
                file_env.update(
                    MIGRATION_DATABASE_URL_FILE=str(secret),
                    DATABASE_URL="mongodb://invalid:1/wrong",
                )
                run([*command, "status"], app, file_env)
                migrate()
                migrate()
                initial = next((app / "migrations/mongo").glob("*.json"))
                original = initial.read_bytes()
                initial.write_bytes(original + b"\n")
                migrate(success=False)
                initial.write_bytes(original)
                admin("setup")
                backfill = app / "migrations/mongo/20261027000200_backfill.json"
                backfill.write_text(
                    json.dumps(
                        {
                            "operation": "copy_field",
                            "source": "message",
                            "target": "display_name",
                            "batch_size": 1,
                        }
                    )
                    + "\n"
                )
                migrate(success=False)
                observed = admin("failed")
                assert observed == {
                    "status": "failed",
                    "checkpoint": 1,
                    "copied": "primeiro",
                }, observed
                admin("fix")
                with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
                    for future in [pool.submit(migrate), pool.submit(migrate)]:
                        future.result()
                complete = admin("complete")
                assert complete["count"] == 3 and complete["lease"] is None, complete
                assert all(row["status"] == "applied" for row in complete["rows"])
                assert (
                    next(
                        row["checkpoint"]
                        for row in complete["rows"]
                        if row["_id"] == "20261027000200"
                    )
                    == 3
                )
                admin("reserve")
                assert migrate(["status"])["reserved_by"] == "test-stale"
                blocked = subprocess.Popen(
                    [*command, "up"],
                    cwd=app,
                    env=app_env,
                    stdin=subprocess.DEVNULL,
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                    start_new_session=True,
                )
                try:
                    time.sleep(0.3)
                    assert blocked.poll() is None
                    assert admin("reserved") == "test-stale"
                finally:
                    if blocked.poll() is None:
                        os.killpg(blocked.pid, signal.SIGTERM)
                        try:
                            blocked.wait(timeout=10)
                        except subprocess.TimeoutExpired:
                            os.killpg(blocked.pid, signal.SIGKILL)
                            blocked.wait()
                assert admin("reserved") == "test-stale"
                migrate(["unlock", "wrong-owner", "--confirm-stopped"], success=False)
                assert admin("reserved") == "test-stale"
                migrate(["unlock", "test-stale", "--confirm-stopped"])
                assert admin("reserved") is None
                print(
                    f"OK {stack}: ledger, checksum, falha parcial, checkpoint, retomada, concorrência e unlock controlado",
                    flush=True,
                )
            finally:
                admin("drop")
    print("Executores nativos validados; somente coleções próprias foram removidas")


if __name__ == "__main__":
    main()
