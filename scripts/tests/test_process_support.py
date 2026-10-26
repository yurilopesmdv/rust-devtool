import os
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path

from ferramentas.boilerplate.scripts.process_support import capture


class ProcessSupportTest(unittest.TestCase):
    def test_success_returns_output(self):
        result = capture(
            [sys.executable, "-c", "print('ok')"],
            Path.cwd(),
            os.environ.copy(),
            timeout=2,
        )
        self.assertEqual((result.returncode, result.stdout), (0, "ok\n"))

    def test_timeout_and_interrupt_stop_grandchildren(self):
        for interrupt in [False, True]:
            with tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                pidfile = root / "pid"
                code = (
                    "import subprocess,time; from pathlib import Path; "
                    "child=subprocess.Popen(['sleep','60']); "
                    f"Path({str(pidfile)!r}).write_text(str(child.pid)); time.sleep(60)"
                )
                if interrupt:
                    script = (
                        "from ferramentas.boilerplate.scripts.process_support import capture; "
                        "import os; from pathlib import Path; "
                        f"capture({[sys.executable, '-c', code]!r}, Path({str(root)!r}), os.environ.copy())"
                    )
                    process = subprocess.Popen(
                        [sys.executable, "-c", script],
                        stdout=subprocess.PIPE,
                        stderr=subprocess.PIPE,
                    )
                    deadline = time.monotonic() + 5
                    while not pidfile.exists() and time.monotonic() < deadline:
                        time.sleep(0.01)
                    self.assertTrue(pidfile.exists())
                    process.send_signal(signal.SIGINT)
                    _, error = process.communicate(timeout=5)
                    self.assertNotEqual(process.returncode, 0)
                    self.assertIn(b"interrompido", error)
                else:
                    with self.assertRaisesRegex(RuntimeError, "timeout"):
                        capture(
                            [sys.executable, "-c", code],
                            root,
                            os.environ.copy(),
                            timeout=0.2,
                        )
                stat = Path(f"/proc/{pidfile.read_text()}/stat")
                self.assertTrue(not stat.exists() or stat.read_text().split()[2] == "Z")
