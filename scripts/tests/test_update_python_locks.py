import os
import subprocess
import sys
import unittest
from pathlib import Path

from ferramentas.boilerplate.scripts.lock_support import normalize_project


class PythonLockMaintenanceTest(unittest.TestCase):
    def test_normalizes_only_fixture_identity(self):
        source = 'name = "python-lock-fixture"\nversion = "0.1.0"\n'
        self.assertEqual(
            normalize_project(source, "python-lock-fixture"),
            'name = "{{PROJECT_NAME}}"\nversion = "0.1.0"\n',
        )
        for invalid in ["missing", source + source]:
            with self.assertRaises(ValueError):
                normalize_project(invalid, "python-lock-fixture")

    def test_check_rejects_upgrade_before_invoking_tools(self):
        script = (
            Path(__file__).resolve().parents[2]
            / "ferramentas/boilerplate/scripts/update_python_locks.py"
        )
        env = os.environ.copy()
        env["PATH"] = ""
        result = subprocess.run(
            [sys.executable, str(script), "--check", "--upgrade"],
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn("--upgrade exige --write", result.stderr)

    def test_base_snapshot_matches_combination_without_integrations(self):
        package = Path(__file__).resolve().parents[2] / "ferramentas/boilerplate"
        self.assertEqual(
            (package / "templates/python-fastapi/uv.lock").read_bytes(),
            (package / "integrations/python-locks/none-0-0.lock").read_bytes(),
        )
