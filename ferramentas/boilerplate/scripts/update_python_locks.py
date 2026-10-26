#!/usr/bin/env python3
"""Check/write uv snapshots using the Python adapter's dependency composer."""

import argparse
import json
import os
import tempfile
from pathlib import Path

if __package__:
    from .lock_support import normalize_project, publish, run
else:
    from lock_support import normalize_project, publish, run

FIXTURE_NAME = "python-lock-fixture"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument(
        "--check", action="store_true", help="Comparar sem sobrescrever snapshots"
    )
    mode.add_argument(
        "--write", action="store_true", help="Gravar após resolver todas as combinações"
    )
    parser.add_argument(
        "--offline", action="store_true", help="Exigir caches Cargo/uv preenchidos"
    )
    parser.add_argument(
        "--upgrade",
        action="store_true",
        help="Com --write, atualizar transitivas compatíveis",
    )
    args = parser.parse_args()
    if args.upgrade and not args.write:
        parser.error("--upgrade exige --write; o check preserva as versões existentes")
    repo = Path(__file__).resolve().parents[3]
    package = repo / "ferramentas/boilerplate"
    env = os.environ.copy()
    if args.offline:
        env.update(CARGO_NET_OFFLINE="true", UV_OFFLINE="true")
    if args.write:
        (package / "integrations/python-locks").mkdir(parents=True, exist_ok=True)
    run(["uv", "--version"], repo, env)
    manifests = json.loads(
        run(
            [
                "cargo",
                "run",
                "--quiet",
                "--locked",
                "-p",
                "rust-boilerplate",
                "--example",
                "python_lock_manifests",
            ],
            repo,
            env,
        )
    )
    snapshots = {}
    with tempfile.TemporaryDirectory(prefix="boilerplate-python-locks-") as temporary:
        for key, manifest in sorted(manifests.items()):
            path = package / f"integrations/python-locks/{key}.lock"
            app = Path(temporary) / key
            app.mkdir()
            (app / "pyproject.toml").write_text(manifest, encoding="utf-8")
            (app / ".python-version").write_text("3.13\n", encoding="utf-8")
            if path.exists():
                (app / "uv.lock").write_text(
                    path.read_text(encoding="utf-8").replace(
                        "{{PROJECT_NAME}}", FIXTURE_NAME
                    ),
                    encoding="utf-8",
                )
            run(["uv", "lock", *(["--upgrade"] if args.upgrade else [])], app, env)
            run(["uv", "sync", "--locked"], app, env)
            snapshots[path] = normalize_project(
                (app / "uv.lock").read_text(encoding="utf-8"), FIXTURE_NAME
            )
            if key == "none-0-0":
                snapshots[package / "templates/python-fastapi/uv.lock"] = snapshots[
                    path
                ]
    changed = publish(snapshots, args.write)
    for path in changed:
        print(f"{'WRITE' if args.write else 'DIFF'} {path.relative_to(repo)}")
    print(f"{len(manifests)} combinações Python; {len(changed)} diferenças.")
    return int(bool(changed) and not args.write)


if __name__ == "__main__":
    raise SystemExit(main())
