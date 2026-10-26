#!/usr/bin/env python3
"""Validate generated sources in temporary directories without starting infrastructure."""

import argparse
import json
import os
import shutil
import tempfile
from pathlib import Path

from process_support import capture


def run(command, cwd, env):
    result = capture(command, cwd, env)
    if result.returncode:
        diagnostic = (result.stdout + result.stderr)[-5000:]
        for key, value in env.items():
            if value and any(
                part in key for part in ["URL", "PASSWORD", "TOKEN", "SECRET"]
            ):
                diagnostic = diagnostic.replace(value, "<redacted>")
        raise RuntimeError(f"{' '.join(command)} falhou em {cwd}:\n{diagnostic}")
    print(f"OK {' '.join(command)} ({cwd.name})", flush=True)
    return result.stdout


def validate_app(app, template, args, env):
    manager = template["package_manager"]
    if manager == "npm":
        run(["npm", "ci", "--no-audit", "--no-fund"], app, env)
        for script in ["lint", "format:check", "typecheck", "build"]:
            run(["npm", "run", script], app, env)
        if args.with_services and template["role"] == "backend":
            run(["npm", "run", "db:prepare"], app, env)
        run(["npm", "test"], app, env)
    elif manager == "go":
        snapshots = {
            name: (app / name).read_bytes() if (app / name).exists() else None
            for name in ["go.mod", "go.sum"]
        }
        run(["go", "mod", "download"], app, env)
        run(["go", "mod", "verify"], app, env)
        if run(["gofmt", "-l", "."], app, env).strip():
            raise RuntimeError("Fontes Go fora do formato")
        if args.with_services and env.get("DATABASE_URL"):
            run(["go", "run", "./cmd/prepare"], app, env)
        run(["go", "test", "-race", "./..."], app, env)
        run(["go", "vet", "./..."], app, env)
        run(["go", "build", "-buildvcs=false", "-o", "bin/api", "./cmd/api"], app, env)
        for name, content in snapshots.items():
            actual = (app / name).read_bytes() if (app / name).exists() else None
            if actual != content:
                raise RuntimeError(f"Validação Go alterou snapshot {name}")
    elif manager == "cargo":
        run(["cargo", "fmt", "--check"], app, env)
        run(["cargo", "test", "--locked"], app, env)
        run(
            ["cargo", "clippy", "--locked", "--all-targets", "--", "-D", "warnings"],
            app,
            env,
        )
        if args.with_services:
            run(["cargo", "test", "--locked", "--", "--ignored"], app, env)
    elif manager == "uv":
        run(["uv", "sync", "--locked"], app, env)
        run(["uv", "run", "--locked", "ruff", "check", "."], app, env)
        run(["uv", "run", "--locked", "ruff", "format", "--check", "."], app, env)
        if args.with_services:
            run(
                ["uv", "run", "--locked", "python", "-m", "app.infrastructure.prepare"],
                app,
                env,
            )
        run(["uv", "run", "--locked", "pytest"], app, env)
    else:
        raise RuntimeError(f"Adicione um validador para o gerenciador {manager}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--with-services",
        action="store_true",
        help="Usar bancos/cache dedicados via *_TEST_URL",
    )
    parser.add_argument(
        "--commonjs-only",
        action="store_true",
        help="Somente Express/CommonJS; --minimal omite banco e infraestrutura",
    )
    parser.add_argument(
        "--stacks", nargs="+", help="IDs do catálogo; padrão: todas as bases prontas"
    )
    parser.add_argument(
        "--minimal", action="store_true", help="Validar as bases sem módulos opcionais"
    )
    parser.add_argument(
        "--keep", action="store_true", help="Preservar fontes temporários"
    )
    parser.add_argument(
        "--offline", action="store_true", help="Exigir caches npm/Cargo/uv preenchidos"
    )
    args = parser.parse_args()
    if args.with_services and args.minimal:
        parser.error("--minimal não usa serviços")
    if args.commonjs_only and args.stacks:
        parser.error("Use --commonjs-only ou --stacks")
    repo = Path(__file__).resolve().parents[3]
    env = os.environ.copy()
    if args.offline:
        env.update(
            CARGO_NET_OFFLINE="true",
            npm_config_offline="true",
            UV_OFFLINE="true",
            GOPROXY="off",
        )
    run(["cargo", "build", "-p", "rust-boilerplate", "--locked"], repo, env)
    metadata = json.loads(
        run(["cargo", "metadata", "--no-deps", "--format-version=1"], repo, env)
    )
    binary = str(Path(metadata["target_directory"]) / "debug/boilerplate")
    templates = json.loads(run([binary, "templates", "list", "--json"], repo, env))
    selected = ["express-ts"] if args.commonjs_only else args.stacks
    if selected and set(selected) - {t["id"] for t in templates}:
        parser.error("Stack desconhecida; consulte boilerplate templates list")
    templates = [
        t
        for t in templates
        if t["status"] == "ready" and (not selected or t["id"] in selected)
    ]
    if args.with_services:
        for capability, key in [
            ("postgres", "POSTGRES_TEST_URL"),
            ("mongodb", "MONGODB_TEST_URL"),
            ("redis", "REDIS_TEST_URL"),
        ]:
            if any(
                capability in template["capabilities"] for template in templates
            ) and not env.get(key):
                parser.error(f"Defina {key} para um serviço de TESTE dedicado")
    if any(t["package_manager"] == "npm" for t in templates) and not run(
        ["node", "--version"], repo, env
    ).strip().startswith("v24."):
        parser.error("Carregue Node 24: nvm use 24")
    root = Path(tempfile.mkdtemp(prefix="boilerplate-validation-"))
    env["CARGO_TARGET_DIR"] = os.environ.get(
        "CARGO_TARGET_DIR", str(root / "cargo-target")
    )
    env["npm_config_cache"] = os.environ.get(
        "npm_config_cache",
        str(Path.home() / ".npm") if args.offline else str(root / "npm-cache"),
    )
    try:
        for template in templates:
            backend = template["role"] == "backend"
            for database in (
                (
                    [
                        db
                        for db in ["postgres", "mongodb"]
                        if db in template["capabilities"]
                    ]
                    or ["none"]
                )
                if backend and not args.minimal
                else ["none"]
            ):
                name = (
                    template["id"]
                    + "-"
                    + database
                    + ("-commonjs" if args.commonjs_only else "")
                )
                app = root / name
                command = [
                    binary,
                    "new",
                    name,
                    "--type",
                    template["role"],
                    "--backend" if backend else "--frontend",
                    template["id"],
                    "--destination",
                    str(app),
                    "--module-system",
                    "commonjs" if args.commonjs_only else "esm",
                    "--install",
                    "no",
                    "--git",
                    "no",
                    "--github",
                    "no",
                    "--non-interactive",
                ]
                if backend and not args.minimal:
                    capabilities = template["capabilities"]
                    if database != "none":
                        command.extend(
                            ["--database", database, "--database-location", "local"]
                        )
                    for capability, flags in [
                        ("redis", ["--cache", "redis", "--cache-location", "local"]),
                        ("compose", ["--containers", "compose"]),
                        ("kubernetes", ["--kubernetes", "manifests"]),
                        ("prometheus-grafana", ["--monitoring", "prometheus-grafana"]),
                        ("vault", ["--secrets", "vault", "--vault-location", "local"]),
                    ]:
                        if capability in capabilities:
                            command.extend(flags)
                run(command, root, env)
                app_env = env.copy()
                if args.with_services and backend and database != "none":
                    app_env["DATABASE_URL"] = env[
                        "POSTGRES_TEST_URL"
                        if database == "postgres"
                        else "MONGODB_TEST_URL"
                    ]
                    if "redis" in template["capabilities"]:
                        app_env["REDIS_URL"] = env["REDIS_TEST_URL"]
                validate_app(app, template, args, app_env)
                if (app / "compose.yaml").exists() and shutil.which("docker"):
                    clean = env.copy()
                    clean.update(
                        POSTGRES_PASSWORD="test-only",
                        MONGO_PASSWORD="test-only",
                        REDIS_PASSWORD="test-only",
                        GRAFANA_ADMIN_PASSWORD="test-only",
                    )
                    run(
                        [
                            "docker",
                            "compose",
                            "--env-file",
                            "infra/.env.example",
                            "config",
                            "--quiet",
                        ],
                        app,
                        clean,
                    )
                if (app / "infra/kubernetes").exists() and shutil.which("kubectl"):
                    run(
                        ["kubectl", "kustomize", "infra/kubernetes/overlays/dev"],
                        app,
                        env,
                    )
        print("Validação concluída. Nenhum container/cluster/remote foi criado.")
    finally:
        if args.keep:
            print(f"Fontes de revisão: {root}")
        else:
            shutil.rmtree(root)


if __name__ == "__main__":
    main()
