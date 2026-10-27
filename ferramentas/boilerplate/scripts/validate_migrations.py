#!/usr/bin/env python3
"""Exercise generated SQL migration runners against an explicitly disposable database."""

import argparse
import concurrent.futures
import json
import os
import tempfile
from pathlib import Path

if __package__:
    from .lock_support import run
    from .process_support import capture
else:
    from lock_support import run
    from process_support import capture


STACKS = ["express-ts", "nest-ts", "rust-axum", "python-fastapi", "go-http"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--disposable-database", action="store_true", required=True)
    parser.add_argument("--stacks", nargs="+", choices=STACKS, default=STACKS)
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[3]
    env = os.environ.copy()
    url = env.get("MIGRATIONS_TEST_URL")
    if not url or not env.get("DBMATE_BIN"):
        parser.error(
            "Defina MIGRATIONS_TEST_URL (banco vazio descartável) e DBMATE_BIN fixado"
        )
    for key in ["DATABASE_URL", "DATABASE_URL_FILE", "MIGRATION_DATABASE_URL_FILE"]:
        env.pop(key, None)
    env.update(BOILERPLATE_MIGRATION_URL=url, MIGRATION_DATABASE_URL=url)
    run(["cargo", "build", "-p", "rust-boilerplate", "--locked"], repo, env)
    metadata = json.loads(
        run(["cargo", "metadata", "--no-deps", "--format-version=1"], repo, env)
    )
    binary = str(Path(metadata["target_directory"]) / "debug/boilerplate")
    helper = repo / "ferramentas/boilerplate/integrations/migrations/pg_client.py"

    def sql(query):
        return run(
            [
                "python3",
                str(helper),
                "--no-psqlrc",
                "-At",
                "-v",
                "ON_ERROR_STOP=1",
                "-c",
                query,
            ],
            repo,
            env,
        ).strip()

    tables = [
        "examples",
        "schema_migrations",
        "boilerplate_migration_checksums",
        "migration_probe",
    ]
    for table in tables:
        if sql(f"SELECT to_regclass('public.{table}') IS NULL") != "t":
            parser.error(
                f"Banco não vazio: {table}; teste recusado antes de alterar dados"
            )
    with tempfile.TemporaryDirectory(
        prefix="boilerplate migrations-validation-"
    ) as temporary:
        root = Path(temporary)
        for stack in args.stacks:
            app = root / stack
            run(
                [
                    binary,
                    "new",
                    "migration-api",
                    "--type",
                    "backend",
                    "--backend",
                    stack,
                    "--database",
                    "postgres",
                    "--database-location",
                    "external",
                    "--migrations",
                    "dbmate",
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

            def migrate(action="up", success=True, app=app, stack=stack):
                result = capture(
                    ["sh", "scripts/migrate.sh", action], app, env, timeout=120
                )
                if (result.returncode == 0) != success:
                    raise RuntimeError(
                        f"Resultado inesperado em {stack}/{action}; código {result.returncode}"
                    )
                if url in result.stdout + result.stderr:
                    raise RuntimeError("Diagnóstico expôs URI")
                return result.stdout

            try:
                migrate("install")
                assert "pending" in migrate("status")
                secret = root / "postgres-url"
                secret.write_text(url)
                secret.chmod(0o600)
                file_env = env.copy()
                file_env.pop("MIGRATION_DATABASE_URL", None)
                file_env.update(
                    MIGRATION_DATABASE_URL_FILE=str(secret),
                    DATABASE_URL="postgresql://invalid:1/wrong",
                )
                run(["sh", "scripts/migrate.sh", "status"], app, file_env)
                migrate()
                migrate()
                assert (
                    sql("SELECT count(*) FROM boilerplate_migration_checksums") == "1"
                )
                initial = next((app / "migrations/sql").glob("*.sql"))
                source = initial.read_bytes()
                initial.write_bytes(source + b"\n-- changed after application\n")
                migrate(success=False)
                initial.write_bytes(source)
                assert sql("SELECT count(*) FROM schema_migrations") == "1"
                second = app / "migrations/sql/20261027000200_add_probe.sql"
                second.write_text(
                    "-- migrate:up\nCREATE TABLE migration_probe (id integer PRIMARY KEY);\nSELECT 1/0;\n-- migrate:down\n"
                )
                migrate(success=False)
                assert (
                    sql("SELECT to_regclass('public.migration_probe') IS NULL") == "t"
                )
                assert sql("SELECT count(*) FROM schema_migrations") == "1"
                second.write_text(
                    "-- migrate:up\nCREATE TABLE migration_probe (id integer PRIMARY KEY);\n-- migrate:down\n"
                )
                migrate()
                third = app / "migrations/sql/20261027000300_concurrent.sql"
                third.write_text(
                    "-- migrate:up\nSELECT pg_sleep(0.5);\nINSERT INTO migration_probe VALUES (1);\n-- migrate:down\n"
                )
                with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
                    futures = [pool.submit(migrate) for _ in range(2)]
                    for future in futures:
                        future.result()
                assert sql("SELECT count(*) FROM migration_probe") == "1"
                assert (
                    sql("SELECT count(*) FROM boilerplate_migration_checksums") == "3"
                )
                assert "pending" not in migrate("status")
                print(
                    f"OK {stack}: up idempotente, checksum, falha transacional, retomada e concorrência",
                    flush=True,
                )
            finally:
                # Initial emptiness check ensures these tables belong only to this test.
                sql(
                    "DROP TABLE IF EXISTS migration_probe, examples, schema_migrations, boilerplate_migration_checksums"
                )
    print("Migrations validadas; tabelas próprias removidas e nenhum serviço iniciado")


if __name__ == "__main__":
    main()
