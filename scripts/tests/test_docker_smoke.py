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
