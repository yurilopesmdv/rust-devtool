#!/usr/bin/env python3
"""Describe validation required by changed dependency files; never update/install."""

import argparse
import json
import subprocess


def review(paths):
    scopes = set()
    for path in paths:
        if path in ["Cargo.toml", "Cargo.lock"] or (
            path.endswith("Cargo.toml")
            and not path.startswith("ferramentas/boilerplate/templates/")
        ):
            scopes.add("workspace")
        if path.startswith(".github/"):
            scopes.add("actions")
        if path.startswith("ferramentas/boilerplate/"):
            if any(
                word in path
                for word in ["package.json", "package-lock.json", "node/", "node.rs"]
            ):
                scopes.add("npm-templates")
            if any(
                word in path
                for word in ["pyproject.toml", "uv.lock", "python-locks/", "python.rs"]
            ):
                scopes.add("python-templates")
            if any(
                word in path
                for word in [
                    "templates/rust-axum/",
                    "rust-locks/",
                    "integrations/rust.rs",
                ]
            ):
                scopes.add("rust-templates")
    checks = {
        "workspace": [
            "sh scripts/check-workspace.sh",
            "Revisar advisories e licenças do grafo Cargo",
        ],
        "actions": [
            "actionlint .github/workflows/quality.yml",
            "Verificar SHA, origem, permissões e dependabot diff",
        ],
        "npm-templates": [
            "python3 ferramentas/boilerplate/scripts/update_node_locks.py --check",
            "python3 ferramentas/boilerplate/scripts/validate-generated.py --minimal --stacks vanilla-js react-vite-ts next-ts express-ts nest-ts",
            "python3 ferramentas/boilerplate/scripts/validate-generated.py --minimal --commonjs-only",
            "Matriz --with-services dos adapters alterados; revisar npm audit e licenças",
        ],
        "python-templates": [
            "python3 ferramentas/boilerplate/scripts/update_python_locks.py --check",
            "python3 ferramentas/boilerplate/scripts/validate-generated.py --minimal --stacks python-fastapi",
            "Matriz banco/cache/métricas/Vault afetada; revisar advisories e licenças Python",
        ],
        "rust-templates": [
            "python3 ferramentas/boilerplate/scripts/update_rust_locks.py --check",
            "python3 ferramentas/boilerplate/scripts/validate-generated.py --minimal --stacks rust-axum",
            "Matriz --with-services afetada; revisar advisories e licenças Rust",
        ],
    }
    return {
        "schema_version": 1,
        "scopes": sorted(scopes),
        "checks": {scope: checks[scope] for scope in sorted(scopes)},
        "note": "Plano de revisão; nenhum check executado e nenhum pacote atualizado",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--files", nargs="+", help="Simular caminhos; padrão: diff do Git contra HEAD"
    )
    parser.add_argument("--base", default="HEAD")
    args = parser.parse_args()
    paths = args.files
    if paths is None:
        result = subprocess.run(
            ["git", "diff", "--name-only", "-z", args.base, "--"],
            capture_output=True,
            check=True,
        )
        paths = result.stdout.decode().split("\0")
    print(json.dumps(review(paths), indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
