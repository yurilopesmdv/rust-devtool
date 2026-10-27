#!/usr/bin/env python3
"""Exercise old/new generated APIs through expand, interrupted backfill and explicit contract."""

import argparse
import json
import os
import signal
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request
from pathlib import Path

if __package__:
    from .lock_support import run
    from .process_support import capture
else:
    from lock_support import run
    from process_support import capture

ADMIN = """
let close;let result;const action=process.env.EVOLUTION_ACTION;
try{
 if(process.env.EVOLUTION_DATABASE==='postgres'){
  const {Pool}=await import('pg');const pool=new Pool({connectionString:process.env.EVOLUTION_TEST_URL});close=()=>pool.end();
  if(action==='empty')result=(await pool.query("SELECT to_regclass('public.examples') IS NULL AND to_regclass('public.schema_migrations') IS NULL AND to_regclass('public.boilerplate_migration_checksums') IS NULL AS empty")).rows[0].empty;
  else if(action==='rolled_back')result=(await pool.query("SELECT display_name IS NULL AS empty FROM examples WHERE key='default'")).rows[0].empty;
  else if(action==='dropdefault'){await pool.query("DELETE FROM examples WHERE key='default'");result=true;}
  else if(action==='drop'){await pool.query('DROP TABLE IF EXISTS examples,schema_migrations,boilerplate_migration_checksums');result=true;}
  else throw new Error('unsupported action');
 }else{
  const {MongoClient}=await import('mongodb');const client=new MongoClient(process.env.EVOLUTION_TEST_URL,{serverSelectionTimeoutMS:5000,socketTimeoutMS:5000});close=()=>client.close();await client.connect();const db=client.db();
  if(action==='empty')result=(await db.listCollections().toArray()).length===0;
  else if(action==='setup'){await db.collection('examples').insertMany([{_id:'a_valid',message:'primeiro'},{_id:'b_invalid',message:123}]);result=true;}
  else if(action==='failed'){const row=await db.collection('_boilerplate_migrations').findOne({_id:'20261027000200'});result=row?.status==='failed'&&Number(row?.checkpoint)===1;}
  else if(action==='fix'){await db.collection('examples').updateOne({_id:'b_invalid'},{$set:{message:'corrigido'}});result=true;}
  else if(action==='dropdefault'){await db.collection('examples').deleteOne({_id:'default'});result=true;}
  else if(action==='drop'){for(const name of ['examples','_boilerplate_migrations','_boilerplate_migration_lock'])if(await db.listCollections({name}).hasNext())await db.collection(name).drop();result=true;}
  else throw new Error('unsupported action');
 }
 console.log(JSON.stringify(result));
}finally{if(close)await close();}
"""


def request(port, message=None, expected=200):
    url = f"http://127.0.0.1:{port}/api/v1/example"
    req = urllib.request.Request(url)
    if message is not None:
        req = urllib.request.Request(
            url,
            data=json.dumps({"message": message}).encode(),
            headers={"Content-Type": "application/json"},
        )
    try:
        response = urllib.request.urlopen(req, timeout=5)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        assert response.status == expected, (response.status, expected)
        return json.loads(response.read())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--disposable-database", action="store_true", required=True)
    parser.add_argument("--database", choices=["postgres", "mongodb"], required=True)
    parser.add_argument("--offline", action="store_true")
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[3]
    fixtures = repo / "ferramentas/boilerplate/fixtures/schema-evolution"
    env = os.environ.copy()
    url = env.get("EVOLUTION_TEST_URL")
    if not url or (args.database == "postgres" and not env.get("DBMATE_BIN")):
        parser.error(
            "Defina EVOLUTION_TEST_URL vazio/descartável e DBMATE_BIN para PostgreSQL"
        )
    for key in ["DATABASE_URL_FILE", "MIGRATION_DATABASE_URL_FILE"]:
        env.pop(key, None)
    env.update(
        DATABASE_URL=url, MIGRATION_DATABASE_URL=url, EVOLUTION_DATABASE=args.database
    )
    if args.offline:
        env.update(CARGO_NET_OFFLINE="true", npm_config_offline="true")
    run(["cargo", "build", "--locked", "-p", "rust-boilerplate"], repo, env)
    metadata = json.loads(
        run(["cargo", "metadata", "--no-deps", "--format-version=1"], repo, env)
    )
    binary = str(Path(metadata["target_directory"]) / "debug/boilerplate")
    if not run(["node", "--version"], repo, env).strip().startswith("v24."):
        parser.error("Use Node 24 no PATH do verificador (carregue nvm use 24)")
    processes = []
    logs = []
    with tempfile.TemporaryDirectory(prefix="schema evolution-") as temporary:
        root = Path(temporary)
        old, new = root / "old", root / "new"
        for app in [old, new]:
            run(
                [
                    binary,
                    "new",
                    app.name + "-api",
                    "--type",
                    "backend",
                    "--backend",
                    "express-ts",
                    "--database",
                    args.database,
                    "--database-location",
                    "external",
                    "--migrations",
                    "dbmate" if args.database == "postgres" else "mongodb-native",
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
            if app == new:
                (
                    app / "src/modules/example/repositories/example.repository.ts"
                ).write_bytes(
                    (fixtures / (args.database + "-new-repository.ts")).read_bytes()
                )
            run(["npm", "ci", "--no-audit", "--no-fund"], app, env)
            for script in ["lint", "format:check", "typecheck", "build"]:
                run(["npm", "run", script], app, env)

        def admin(action):
            control = env.copy()
            control["EVOLUTION_ACTION"] = action
            return json.loads(
                run(["node", "--input-type=module", "-e", ADMIN], old, control)
            )

        if not admin("empty"):
            parser.error("Banco não vazio; ensaio recusado antes de alterar dados")
        command = (
            ["sh", "scripts/migrate.sh"]
            if args.database == "postgres"
            else ["node", "scripts/migrate.mjs"]
        )

        def migrate(success=True):
            result = capture([*command, "up"], old, env, timeout=120)
            if (result.returncode == 0) != success:
                raise RuntimeError(
                    f"Migration inesperada em {args.database}: {result.returncode}"
                )
            if url in result.stdout + result.stderr:
                raise RuntimeError("URI exposta no diagnóstico")

        def migration(name, fixture):
            directory = "sql" if args.database == "postgres" else "mongo"
            for app in [old, new]:
                (app / "migrations" / directory / name).write_bytes(
                    (fixtures / fixture).read_bytes()
                )

        def serve(app):
            with socket.socket() as sock:
                sock.bind(("127.0.0.1", 0))
                port = sock.getsockname()[1]
            app_env = env.copy()
            app_env["PORT"] = str(port)
            log = (root / (app.name + str(len(processes)) + ".log")).open("w")
            logs.append(log)
            process = subprocess.Popen(
                ["node", "dist/server.js"],
                cwd=app,
                env=app_env,
                stdout=log,
                stderr=log,
                start_new_session=True,
            )
            processes.append(process)
            deadline = time.monotonic() + 15
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    raise RuntimeError("API encerrou durante startup")
                try:
                    with urllib.request.urlopen(
                        f"http://127.0.0.1:{port}/ready", timeout=1
                    ) as response:
                        if response.status == 200:
                            return process, port
                except (urllib.error.URLError, OSError):
                    time.sleep(0.05)
            raise RuntimeError("API não atingiu readiness")

        def stop(process):
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGTERM)
                try:
                    code = process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait()
                    raise RuntimeError("API não encerrou dentro do limite") from None
                assert code == 0, code

        try:
            if args.database == "postgres":
                run([*command, "install"], old, env)
            migrate()
            old_process, old_port = serve(old)
            request(old_port, "old-before")
            if args.database == "postgres":
                migration("20261027000200_expand.sql", "expand.sql")
                migrate()
            new_process, new_port = serve(new)
            assert request(new_port)["display_name"] == "old-before"
            assert request(old_port)["message"] == "old-before"
            if args.database == "postgres":
                migration("20261027000300_backfill.sql", "backfill-failure.sql")
                migrate(success=False)
                assert admin("rolled_back") is True
                migration("20261027000300_backfill.sql", "backfill.sql")
            else:
                admin("setup")
                migration("20261027000200_backfill.json", "backfill.json")
                migrate(success=False)
                assert admin("failed") is True
                admin("fix")
            # Both releases keep serving through the interrupted migration.
            assert request(old_port)["message"] == "old-before"
            assert request(new_port)["message"] == "old-before"
            migrate()
            migrate()
            assert request(new_port)["display_name"] == "old-before"
            request(old_port, "old-after")
            assert request(new_port)["message"] == "old-after"
            assert request(new_port)["display_name"] == "old-before"
            request(new_port, "new-after")
            assert request(old_port)["message"] == "new-after"
            # The controlled rehearsal explicitly retires the old writer before contract.
            stop(old_process)
            if args.database == "postgres":
                migration("20261027000400_contract.sql", "contract.sql")
            else:
                migration("20261027000300_contract.json", "contract.json")
            migrate()
            request(new_port, "after-contract")
            # Negative rollback replay in this disposable database: old-shaped inserts now fail.
            replay, replay_port = serve(old)
            admin("dropdefault")
            request(replay_port, "old-invalid", expected=500)
            request(new_port, "new-restored")
            assert request(new_port)["display_name"] == "new-restored"
            if args.database == "postgres":
                migration("20261027000500_recovery.sql", "recovery.sql")
            else:
                migration("20261027000400_recovery.json", "recovery.json")
            migrate()
            migrate()
            admin("dropdefault")
            request(replay_port, "old-recovered")
            assert request(new_port)["display_name"] == "old-recovered"
            assert request(replay_port)["message"] == "old-recovered"
            stop(replay)
            stop(new_process)
            print(
                f"OK {args.database}: duas APIs reais, expand, falha/retomada, compatibilidade, contract e recuperação corretiva",
                flush=True,
            )
        finally:
            for process in reversed(processes):
                if process.poll() is None:
                    os.killpg(process.pid, signal.SIGTERM)
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        os.killpg(process.pid, signal.SIGKILL)
                        process.wait()
            for log in logs:
                log.close()
            admin("drop")


if __name__ == "__main__":
    main()
