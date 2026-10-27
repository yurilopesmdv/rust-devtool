#!/usr/bin/env python3
"""Opt-in smoke test against a disposable Linux Docker daemon. Never used by new."""

import argparse
import json
import os
import sys
import tempfile
import time
import urllib.error
import urllib.request
import uuid
from pathlib import Path

if __package__:
    from .process_support import capture
else:
    from process_support import capture


FILES = {
    "express-ts": "/app/dist/server.js",
    "nest-ts": "/app/dist/main.js",
    "rust-axum": "/app/server",
    "go-http": "/app/server",
    "python-fastapi": "/app/app/main.py",
    "next-ts": "/app/server.js",
    "react-vite-ts": "/usr/share/nginx/html/index.html",
    "vanilla-js": "/usr/share/nginx/html/index.html",
}


def run(command, cwd):
    result = capture(command, cwd, os.environ.copy(), timeout=1200)
    if result.returncode:
        raise RuntimeError(
            f"{command[0]} failed: {(result.stdout + result.stderr)[-3000:]}"
        )
    return result.stdout.strip()


def wait_ready(
    url,
    opener=urllib.request.urlopen,
    clock=time.monotonic,
    pause=time.sleep,
    timeout=90,
):
    deadline = clock() + timeout
    last_failure = "sem resposta HTTP"
    while clock() < deadline:
        try:
            with opener(url, timeout=2) as response:
                if response.status == 200:
                    return
        except urllib.error.HTTPError as error:
            last_failure = f"HTTP {error.code}"
        except (urllib.error.URLError, OSError) as error:
            # Startup can reset a socket before the HTTP listener becomes available.
            last_failure = type(error).__name__
        pause(0.5)
    raise RuntimeError(
        f"Imagem não atingiu readiness no limite configurado: {last_failure}"
    )


def smoke(binary, template, root, execute=run, probe=None):
    label = "rust-devtool-smoke-" + uuid.uuid4().hex
    image = label + ":test"
    app = root / "app"
    created = False
    built = False
    try:
        role = (
            "frontend"
            if template in ["vanilla-js", "react-vite-ts", "next-ts"]
            else "backend"
        )
        execute(
            [
                str(binary),
                "new",
                "smoke-app",
                "--type",
                role,
                "--" + role,
                template,
                "--destination",
                str(app),
                "--containers",
                "compose",
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
        execute(
            [
                "docker",
                "build",
                "--label",
                "rust-devtool.smoke=" + label,
                "-t",
                image,
                str(app),
            ],
            root,
        )
        built = True
        port = (
            "8080"
            if template in ["vanilla-js", "react-vite-ts"]
            else "3001"
            if template == "next-ts"
            else "3000"
        )
        execute(
            [
                "docker",
                "create",
                "--name",
                label,
                "--label",
                "rust-devtool.smoke=" + label,
                "--cap-drop",
                "ALL",
                "--security-opt",
                "no-new-privileges:true",
                "-p",
                "127.0.0.1::" + port,
                image,
            ],
            root,
        )
        created = True
        execute(["docker", "start", label], root)
        user = execute(["docker", "exec", label, "id", "-u"], root)
        if not user.isdigit() or int(user) == 0:
            raise RuntimeError("Container precisa executar como usuário não root")
        execute(["docker", "exec", label, "test", "-f", FILES[template]], root)
        address = execute(["docker", "port", label, port + "/tcp"], root).splitlines()[
            0
        ]
        if not address.startswith("127.0.0.1:"):
            raise RuntimeError("Porta do teste precisa estar limitada ao loopback")
        url = "http://" + address + ("/ready" if role == "backend" else "/")
        if probe:
            probe(url)
        else:
            wait_ready(url)
        execute(["docker", "stop", "--time", "5", label], root)
        state = json.loads(
            execute(["docker", "inspect", "--format", "{{json .State}}", label], root)
        )
        if state["Running"] or state["OOMKilled"] or state["ExitCode"] == 137:
            raise RuntimeError("Encerramento não foi limpo")
    finally:
        # Always attempt both cleanups and preserve the primary failure.
        primary_failure = sys.exc_info()[0] is not None
        commands = []
        if created:
            commands.append(["docker", "rm", "-f", label])
        if built:
            commands.append(["docker", "image", "rm", image])
        failures = []
        for command in commands:
            try:
                execute(command, root)
            except (RuntimeError, OSError) as error:
                failures.append(str(error))
        if failures:
            message = "Cleanup incompleto dos recursos próprios: " + "; ".join(failures)
            if primary_failure:
                print(message, file=sys.stderr)
            else:
                raise RuntimeError(message)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--disposable-daemon",
        action="store_true",
        required=True,
        help="Confirmar que o contexto Docker aponta para daemon descartável de teste",
    )
    parser.add_argument("--stack", choices=sorted(FILES), required=True)
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[3]
    if run(["docker", "info", "--format", "{{.OSType}}"], repo) != "linux":
        parser.error("O daemon de teste deve executar containers Linux")
    run(["cargo", "build", "-p", "rust-boilerplate", "--locked"], repo)
    metadata = json.loads(
        run(["cargo", "metadata", "--no-deps", "--format-version=1"], repo)
    )
    binary = Path(metadata["target_directory"]) / "debug/boilerplate"
    with tempfile.TemporaryDirectory(prefix="rust-devtool-docker-smoke-") as temporary:
        smoke(binary, args.stack, Path(temporary))
    print(f"OK Docker smoke: {args.stack}; recursos próprios removidos")


if __name__ == "__main__":
    main()
