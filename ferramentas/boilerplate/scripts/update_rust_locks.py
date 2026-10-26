#!/usr/bin/env python3
"""Check/write all Rust lock snapshots from the generator's own manifest composer."""

import argparse
import json
import os
import re
import tempfile
from pathlib import Path

if __package__:
    from .lock_support import normalize_project, publish, run
else:
    from lock_support import normalize_project, publish, run

FIXTURE_NAME = "rust-lock-fixture"


def normalize(text):
    return normalize_project(text, FIXTURE_NAME)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument(
        "--check", action="store_true", help="Comparar sem sobrescrever snapshots"
    )
    mode.add_argument(
        "--write",
        action="store_true",
        help="Gravar diferenças após resolver todas as combinações",
    )
    parser.add_argument(
        "--offline", action="store_true", help="Usar somente o cache Cargo"
    )
    parser.add_argument(
        "--upgrade",
        action="store_true",
        help="Com --write, atualizar também dependências transitivas compatíveis",
    )
    args = parser.parse_args()
    if args.upgrade and not args.write:
        parser.error("--upgrade exige --write; o check preserva as versões existentes")
    repo = Path(__file__).resolve().parents[3]
    package = repo / "ferramentas/boilerplate"
    env = os.environ.copy()
    if args.offline:
        env["CARGO_NET_OFFLINE"] = "true"
    if args.write:
        # Recovery of deleted snapshots must also work when their directory is absent.
        (package / "integrations/rust-locks").mkdir(parents=True, exist_ok=True)
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
                "rust_lock_manifests",
            ],
            repo,
            env,
        )
    )
    host = next(
        (
            line[len("host: ") :]
            for line in run(["rustc", "-vV"], repo, env).splitlines()
            if line.startswith("host: ")
        ),
        None,
    )
    if not host or "-linux-" not in host:
        raise ValueError("A manutenção exige toolchain Rust Linux")
    snapshots = {}
    with tempfile.TemporaryDirectory(prefix="boilerplate-rust-locks-") as temporary:
        for key, manifest in sorted(manifests.items()):
            path = package / (
                "templates/rust-axum/Cargo.lock"
                if key == "none-0-0"
                else f"integrations/rust-locks/{key}.lock"
            )
            app = Path(temporary) / key
            (app / "src").mkdir(parents=True)
            (app / "src/main.rs").write_text("fn main() {}\n", encoding="utf-8")
            (app / "Cargo.toml").write_text(manifest, encoding="utf-8")
            if path.exists():
                (app / "Cargo.lock").write_text(
                    path.read_text(encoding="utf-8").replace(
                        "{{PROJECT_NAME}}", FIXTURE_NAME
                    ),
                    encoding="utf-8",
                )
            metadata = json.loads(
                run(["cargo", "metadata", "--no-deps", "--format-version=1"], app, env)
            )
            for dependency in metadata["packages"][0]["dependencies"]:
                if not re.fullmatch(
                    r"=\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?", dependency["req"]
                ):
                    raise ValueError(
                        f"{key}: fixe uma versão exata para {dependency['name']}"
                    )
            # Preserve existing transitive versions unless a manifest change requires resolution.
            run(
                ["cargo", "update", *([] if args.upgrade else ["--workspace"])],
                app,
                env,
            )
            run(
                ["cargo", "metadata", "--locked", "--no-deps", "--format-version=1"],
                app,
                env,
            )
            run(["cargo", "fetch", "--locked", "--target", host], app, env)
            snapshots[path] = normalize(
                (app / "Cargo.lock").read_text(encoding="utf-8")
            )
    changed = publish(snapshots, args.write)
    for path in changed:
        print(f"{'WRITE' if args.write else 'DIFF'} {path.relative_to(repo)}")
    print(f"{len(snapshots)} combinações Rust; {len(changed)} diferenças.")
    return int(bool(changed) and not args.write)


if __name__ == "__main__":
    raise SystemExit(main())
