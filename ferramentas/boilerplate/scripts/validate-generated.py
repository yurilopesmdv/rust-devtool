#!/usr/bin/env python3
"""Build generated integrations in temporary directories; never start infrastructure."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


def run(command, cwd, env):
    result = subprocess.run(command, cwd=cwd, env=env, text=True, capture_output=True)
    if result.returncode:
        # Never print the environment or connection URLs.
        raise RuntimeError("{} falhou em {}:\n{}".format(" ".join(command), cwd, (result.stdout + result.stderr)[-5000:]))
    print("OK {} ({})".format(" ".join(command), cwd.name), flush=True)
    return result.stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--with-services", action="store_true", help="Use bancos de teste existentes via POSTGRES_TEST_URL, MONGODB_TEST_URL e REDIS_TEST_URL")
    parser.add_argument("--commonjs-only", action="store_true", help="Validar somente as duas bases Express/CommonJS com dados")
    parser.add_argument("--keep", action="store_true", help="Preservar fontes/logs temporários para revisão")
    parser.add_argument("--offline", action="store_true", help="Exigir caches npm/Cargo já preenchidos")
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[3]
    env = os.environ.copy()
    if args.offline:
        env["CARGO_NET_OFFLINE"]="true"
        env["npm_config_offline"]="true"
    if args.with_services:
        for key in ["POSTGRES_TEST_URL", "MONGODB_TEST_URL", "REDIS_TEST_URL"]:
            if not env.get(key):
                parser.error("Defina {} apontando para um serviço de TESTE dedicado".format(key))
    version = run(["node", "--version"], repo, env).strip()
    if not version.startswith("v24."):
        parser.error("Carregue Node 24 antes de executar: nvm use 24")
    run(["cargo", "build", "-p", "rust-boilerplate", "--locked"], repo, env)
    # Cargo workspaces share target; honor an explicit target-dir rather than guessing.
    metadata = json.loads(run(["cargo", "metadata", "--no-deps", "--format-version=1"], repo, env))
    binary = str(Path(metadata["target_directory"]) / "debug/boilerplate")
    root = Path(tempfile.mkdtemp(prefix="boilerplate-validation-"))
    env["CARGO_TARGET_DIR"] = os.environ.get("CARGO_TARGET_DIR", str(root / "cargo-target"))
    env["npm_config_cache"] = os.environ.get("npm_config_cache", str(Path.home() / ".npm") if args.offline else str(root / "npm-cache"))
    if args.offline:
        env["npm_config_offline"] = "true"
        env["CARGO_NET_OFFLINE"] = "true"
    try:
        for stack in (["express-ts"] if args.commonjs_only else ["express-ts", "nest-ts", "rust-axum"]):
            for database in ["postgres", "mongodb"]:
                name = stack + "-" + database + ("-commonjs" if args.commonjs_only else "")
                destination = root / name
                run([binary, "new", name, "--type", "backend", "--backend", stack,
                     "--destination", str(destination), "--module-system", "commonjs" if args.commonjs_only else "esm", "--database", database,
                     "--database-location", "local", "--cache", "redis", "--cache-location", "local",
                     "--containers", "compose", "--kubernetes", "manifests",
                     "--monitoring", "prometheus-grafana", "--secrets", "vault", "--vault-location", "local",
                     "--install", "no", "--git", "no", "--github", "no", "--non-interactive"], repo, env)
                app_env = env.copy()
                if args.with_services:
                    app_env["DATABASE_URL"] = env["POSTGRES_TEST_URL" if database == "postgres" else "MONGODB_TEST_URL"]
                    app_env["REDIS_URL"] = env["REDIS_TEST_URL"]
                if stack == "rust-axum":
                    run(["cargo", "fmt"], destination, app_env)
                    run(["cargo", "test", "--locked"], destination, app_env)
                    run(["cargo", "clippy", "--locked", "--all-targets", "--", "-D", "warnings"], destination, app_env)
                    if args.with_services:
                        run(["cargo", "test", "--locked", "--", "--ignored"], destination, app_env)
                else:
                    run(["npm", "ci", "--no-audit", "--no-fund"], destination, app_env)
                    run(["npm", "run", "typecheck"], destination, app_env)
                    run(["npm", "run", "build"], destination, app_env)
                    if args.with_services:
                        run(["npm", "run", "db:prepare"], destination, app_env)
                    run(["npm", "test"], destination, app_env)
                if shutil.which("docker"):
                    clean = env.copy()
                    clean.update(POSTGRES_PASSWORD="test-only", MONGO_PASSWORD="test-only", REDIS_PASSWORD="test-only",
                                 DATABASE_URL="postgresql://app:test-only@database:5432/app", REDIS_URL="redis://:test-only@cache:6379",
                                 GRAFANA_ADMIN_PASSWORD="test-only", VAULT_ADDR="http://vault:8200")
                    run(["docker", "compose", "--env-file", "infra/.env.example", "config", "--quiet"], destination, clean)
                if shutil.which("kubectl"):
                    run(["kubectl", "kustomize", "infra/kubernetes/overlays/dev"], destination, env)
        print("Validação concluída. Nenhum container/cluster/remote foi criado.")
    finally:
        if args.keep:
            print("Fontes de revisão: {}".format(root))
        else:
            shutil.rmtree(root)


if __name__ == "__main__":
    main()
