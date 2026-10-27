#!/usr/bin/env python3
"""Validate generated CI for every stack and fullstack layout with actionlint."""

import argparse
import json
import os
import tempfile
from pathlib import Path

from process_support import capture


def run(command, cwd):
    result = capture(command, cwd, os.environ.copy())
    if result.returncode:
        raise RuntimeError(result.stdout + result.stderr)
    return result.stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--actionlint", type=Path, required=True)
    args = parser.parse_args()
    binary = str(args.binary.resolve())
    actionlint = str(args.actionlint.resolve())
    with tempfile.TemporaryDirectory(prefix="boilerplate-ci-") as directory:
        root = Path(directory)
        templates = json.loads(run([binary, "templates", "list", "--json"], root))
        front = [t for t in templates if t["role"] == "frontend"]
        back = [t for t in templates if t["role"] == "backend"]
        cases = [["--type", t["role"], "--" + t["role"], t["id"]] for t in templates]
        for web in front:
            for api in back:
                for layout in ["monorepo", "split"]:
                    cases.append(
                        [
                            "--type",
                            "fullstack",
                            "--layout",
                            layout,
                            "--frontend",
                            web["id"],
                            "--backend",
                            api["id"],
                        ]
                    )
        workflows = 0
        for index, flags in enumerate(cases):
            destination = root / f"case-{index}"
            paths = (
                [
                    "--web-dir",
                    str(destination / "web"),
                    "--api-dir",
                    str(destination / "api"),
                ]
                if "split" in flags
                else ["--destination", str(destination)]
            )
            run(
                [
                    binary,
                    "new",
                    f"case-{index}",
                    *flags,
                    *paths,
                    "--ci",
                    "github-actions",
                    "--install",
                    "no",
                    "--git",
                    "no",
                    "--github",
                    "no",
                    "--non-interactive",
                ],
                root,
            )
            for workflow in destination.rglob(".github/workflows/ci.yml"):
                run([actionlint, "-shellcheck=", str(workflow)], root)
                workflows += 1
        expected = len(templates) + len(front) * len(back) * 3
        if workflows != expected:
            raise RuntimeError(f"Expected {expected} workflows, got {workflows}")
        print(f"OK actionlint: {workflows} workflows / {len(cases)} projects")


if __name__ == "__main__":
    main()
