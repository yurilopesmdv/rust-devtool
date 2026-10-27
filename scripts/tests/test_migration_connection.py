import unittest

from ferramentas.boilerplate.integrations.migrations.pg_client import connection_env


class MigrationConnectionTest(unittest.TestCase):
    def test_percent_encoded_credentials_and_tls_go_only_to_environment(self):
        env = connection_env(
            "postgresql://app:p%40ss%2Fword@[::1]:5433/project?sslmode=verify-full&sslrootcert=%2Fcerts%2Fca.pem",
            {"PGSERVICE": "old", "PGHOSTADDR": "old", "PGOPTIONS": "old"},
        )
        self.assertEqual(env["PGPASSWORD"], "p@ss/word")
        self.assertEqual(env["PGHOST"], "::1")
        self.assertEqual(env["PGPORT"], "5433")
        self.assertEqual(env["PGDATABASE"], "project")
        self.assertEqual(env["PGSSLMODE"], "verify-full")
        self.assertEqual(env["PGSSLROOTCERT"], "/certs/ca.pem")
        for key in ["PGSERVICE", "PGHOSTADDR", "PGOPTIONS"]:
            self.assertNotIn(key, env)

    def test_unsupported_and_duplicate_options_fail_instead_of_downgrading_tls(self):
        for uri in [
            "postgresql://localhost/db?sslmode=require&sslmode=disable",
            "postgresql://localhost/db?search_path=custom",
            "postgresql://one,two/db",
            "postgresql://localhost:70000/db",
            "postgresql://localhost/db#fragment",
            "sqlite:///db",
        ]:
            with self.subTest(uri=uri), self.assertRaises(ValueError):
                connection_env(uri, {})

    def test_unverified_override_is_never_executed(self):
        import os
        import subprocess
        import tempfile
        from pathlib import Path

        source = (
            Path(__file__).resolve().parents[2]
            / "ferramentas/boilerplate/integrations/migrations/migrate.sh"
        )
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "scripts").mkdir()
            script = root / "scripts/migrate.sh"
            script.write_bytes(source.read_bytes())
            binary = root / "unverified"
            marker = root / "executed"
            binary.write_text('#!/bin/sh\ntouch "$MIGRATION_TEST_MARKER"\n')
            binary.chmod(0o700)
            env = os.environ.copy()
            env.update(DBMATE_BIN=str(binary), MIGRATION_TEST_MARKER=str(marker))
            result = subprocess.run(
                ["sh", str(script), "install"],
                env=env,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("SHA-256", result.stderr)
            self.assertFalse(marker.exists())
