import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from ferramentas.boilerplate.scripts.update_rust_locks import normalize, publish


class RustLockMaintenanceTest(unittest.TestCase):
    def test_check_reports_changed_and_missing_files_without_writing(self):
        with tempfile.TemporaryDirectory(prefix="rust-lock-check-") as temp:
            root = Path(temp)
            existing = root / "existing.lock"
            missing = root / "missing.lock"
            existing.write_text("before", encoding="utf-8")
            changed = publish({existing: "after", missing: "new"}, write=False)
            self.assertEqual(changed, [existing, missing])
            self.assertEqual(existing.read_text(), "before")
            self.assertFalse(missing.exists())

    def test_write_is_stable_on_second_publication(self):
        with tempfile.TemporaryDirectory(prefix="rust-lock-write-") as temp:
            target = Path(temp) / "locks/project.lock"
            self.assertEqual(publish({target: "value"}, write=True), [target])
            self.assertEqual(publish({target: "value"}, write=True), [])
            self.assertEqual(target.read_text(), "value")

    def test_check_rejects_upgrade_before_invoking_cargo(self):
        script = (
            Path(__file__).resolve().parents[2]
            / "ferramentas/boilerplate/scripts/update_rust_locks.py"
        )
        env = os.environ.copy()
        env["PATH"] = ""
        result = subprocess.run(
            [sys.executable, str(script), "--check", "--upgrade"],
            check=False,
            capture_output=True,
            text=True,
            env=env,
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn("--upgrade exige --write", result.stderr)

    def test_normalization_changes_only_project_identity(self):
        source = 'name = "rust-lock-fixture"\nversion = "0.1.0"\nchecksum = "example"\n'
        self.assertEqual(
            normalize(source),
            source.replace('name = "rust-lock-fixture"', 'name = "{{PROJECT_NAME}}"'),
        )
        for invalid in ["missing", source + source]:
            with self.assertRaises(ValueError):
                normalize(invalid)
