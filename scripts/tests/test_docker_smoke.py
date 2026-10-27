import json
import tempfile
import unittest
from pathlib import Path

from ferramentas.boilerplate.scripts.docker_smoke import smoke


class DockerSmokeTest(unittest.TestCase):
    def test_owned_cleanup_on_success_and_readiness_failure(self):
        for fail in [False, True]:
            calls = []

            def execute(command, root, calls=calls):
                calls.append(command)
                if command[1:2] == ["exec"] and command[-2:] == ["id", "-u"]:
                    return "1000"
                if command[1:2] == ["port"]:
                    return "127.0.0.1:45678"
                if command[1:2] == ["inspect"]:
                    return json.dumps(
                        {"Running": False, "OOMKilled": False, "ExitCode": 0}
                    )
                return ""

            def probe(url, fail=fail):
                self.assertEqual(url, "http://127.0.0.1:45678/ready")
                if fail:
                    raise RuntimeError("readiness failure")

            with tempfile.TemporaryDirectory() as temporary:
                if fail:
                    with self.assertRaisesRegex(RuntimeError, "readiness failure"):
                        smoke(
                            Path("/tmp/boilerplate"),
                            "express-ts",
                            Path(temporary),
                            execute,
                            probe,
                        )
                else:
                    smoke(
                        Path("/tmp/boilerplate"),
                        "express-ts",
                        Path(temporary),
                        execute,
                        probe,
                    )
            remove = [call for call in calls if call[1:2] == ["rm"]]
            self.assertEqual(len(remove), 1)
            self.assertTrue(remove[0][-1].startswith("rust-devtool-smoke-"))
            self.assertFalse(
                any(
                    "prune" in call or "volume" in call or "-v" in call
                    for call in calls
                )
            )
            self.assertEqual(calls[-1][1:3], ["image", "rm"])

    def test_readiness_retries_connection_reset_and_http_startup_failures(self):
        from contextlib import nullcontext
        from types import SimpleNamespace
        from urllib.error import HTTPError

        from ferramentas.boilerplate.scripts.docker_smoke import wait_ready

        attempts = [
            ConnectionResetError(),
            HTTPError("http://localhost", 503, "startup", {}, None),
            None,
        ]
        calls = []

        def opener(url, timeout):
            calls.append(url)
            result = attempts.pop(0)
            if result is not None:
                raise result
            return nullcontext(SimpleNamespace(status=200))

        wait_ready(
            "http://localhost", opener=opener, clock=lambda: 0, pause=lambda _: None
        )
        self.assertEqual(len(calls), 3)

    def test_readiness_deadline_is_bounded(self):
        from ferramentas.boilerplate.scripts.docker_smoke import wait_ready

        ticks = iter([0, 0, 91])

        def unavailable(*args, **kwargs):
            raise ConnectionResetError()

        with self.assertRaisesRegex(RuntimeError, "readiness"):
            wait_ready(
                "http://localhost",
                opener=unavailable,
                clock=lambda: next(ticks),
                pause=lambda _: None,
            )

    def test_cleanup_failure_does_not_mask_readiness_and_both_removals_are_attempted(
        self,
    ):
        from contextlib import redirect_stderr
        from io import StringIO

        calls = []

        def execute(command, root):
            calls.append(command)
            if command[1:2] == ["exec"] and command[-2:] == ["id", "-u"]:
                return "1000"
            if command[1:2] == ["port"]:
                return "127.0.0.1:45678"
            if command[1:2] == ["rm"]:
                raise RuntimeError("container removal failed")
            return ""

        def probe(url):
            raise RuntimeError("primary readiness failure")

        with tempfile.TemporaryDirectory() as temporary:
            errors = StringIO()
            expected = self.assertRaisesRegex(RuntimeError, "primary readiness failure")
            with redirect_stderr(errors), expected:
                smoke(
                    Path("/tmp/boilerplate"),
                    "express-ts",
                    Path(temporary),
                    execute,
                    probe,
                )
            self.assertIn("Cleanup incompleto", errors.getvalue())
        self.assertEqual(calls[-1][1:3], ["image", "rm"])
