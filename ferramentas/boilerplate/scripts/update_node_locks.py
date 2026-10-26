#!/usr/bin/env python3
"""Maintain npm base/superset snapshots using the generator's manifest composer."""

import argparse
import json
import os
import re
import tempfile
from pathlib import Path

if __package__:
    from .lock_support import publish, run
else:
    from lock_support import publish, run


def normalize(lock, base):
    if base:
        lock["name"] = "{{PROJECT_NAME}}"
        lock["packages"][""]["name"] = "{{PROJECT_NAME}}"
        lock["packages"][""]["engines"]["node"] = "{{NODE_ENGINE}}"
    return json.dumps(lock, indent=2, ensure_ascii=False) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--write", action="store_true")
    parser.add_argument("--offline", action="store_true")
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[3]
    package = repo / "ferramentas/boilerplate"
    env = os.environ.copy()
    if args.offline:
        env.update(CARGO_NET_OFFLINE="true", npm_config_offline="true")
    run(["npm", "--version"], repo, env)
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
                "node_lock_manifests",
            ],
            repo,
            env,
        )
    )
    snapshots = {}
    with tempfile.TemporaryDirectory(prefix="boilerplate-npm-locks-") as temporary:
        for relative, manifest in sorted(manifests.items()):
            for group in ["dependencies", "devDependencies"]:
                for name, version in manifest.get(group, {}).items():
                    if not re.fullmatch(r"\d+\.\d+\.\d+(?:-[\w.-]+)?", version):
                        raise ValueError(
                            f"Versão direta precisa ser exata: {name}={version}"
                        )
            destination = package / relative
            app = Path(temporary) / relative.replace("/", "-")
            app.mkdir()
            (app / "package.json").write_text(json.dumps(manifest, indent=2) + "\n")
            if destination.exists():
                lock = json.loads(destination.read_text(encoding="utf-8"))
                lock["name"] = manifest["name"]
                lock["packages"][""]["name"] = manifest["name"]
                lock["packages"][""]["engines"] = manifest["engines"]
                (app / "package-lock.json").write_text(
                    json.dumps(lock, indent=2) + "\n"
                )
            run(
                [
                    "npm",
                    "install",
                    "--package-lock-only",
                    "--ignore-scripts",
                    "--no-audit",
                    "--no-fund",
                ],
                app,
                env,
            )
            run(["npm", "ci", "--ignore-scripts", "--no-audit", "--no-fund"], app, env)
            lock = json.loads((app / "package-lock.json").read_text())
            snapshots[destination] = normalize(lock, relative.startswith("templates/"))
    changed = publish(snapshots, args.write)
    for path in changed:
        print(f"{'WRITE' if args.write else 'DIFF'} {path.relative_to(repo)}")
    print(f"{len(manifests)} snapshots npm; {len(changed)} diferenças.")
    return int(bool(changed) and not args.write)


if __name__ == "__main__":
    raise SystemExit(main())
