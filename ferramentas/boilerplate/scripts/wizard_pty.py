#!/usr/bin/env python3
"""Exercise the real interactive wizard in an isolated Linux/WSL PTY."""

import argparse
import errno
import os
import pty
import select
import shutil
import subprocess
import tempfile
import time
from pathlib import Path


def session(binary, cancel, backend=False, postgres=False):
    with tempfile.TemporaryDirectory(prefix="boilerplate-pty-") as temporary:
        root = Path(temporary)
        env = os.environ.copy()
        env.update(HOME=str(root), PATH="/nonexistent", NVM_DIR=str(root / "nvm"))
        if backend:
            gofmt = shutil.which("gofmt")
            if not gofmt:
                raise RuntimeError("Carregue gofmt antes de testar o assistente Go")
            env["PATH"] = str(Path(gofmt).parent)
        master, slave = pty.openpty()
        command = [
            str(binary),
            "new",
            "--backend" if backend else "--frontend",
            "go-http" if backend else "vanilla-js",
            "--node-version",
            "24",
            "--module-system",
            "esm",
            "--containers",
            "none",
            "--kubernetes",
            "none",
            *([] if backend else ["--monitoring", "none", "--secrets", "env"]),
            "--frontend-origin" if backend else "--api-url",
            "http://localhost:3000",
            "--git",
            "no",
            "--github",
            "no",
            "--install",
            "no",
        ]
        process = subprocess.Popen(
            command,
            cwd=root,
            env=env,
            stdin=slave,
            stdout=slave,
            stderr=slave,
            start_new_session=True,
        )
        os.close(slave)
        buffer = b""

        def expect(marker):
            nonlocal buffer
            target = marker.encode()
            deadline = time.monotonic() + 10
            while target not in buffer:
                if time.monotonic() >= deadline:
                    raise AssertionError(f"Prompt ausente: {marker}; saída: {buffer!r}")
                if select.select([master], [], [], 0.1)[0]:
                    try:
                        data = os.read(master, 65536)
                    except OSError as error:
                        if error.errno != errno.EIO:
                            raise
                        data = b""
                    if not data:
                        raise AssertionError(
                            f"PTY encerrada antes de {marker}: {buffer!r}"
                        )
                    buffer += data
            before, buffer = buffer.split(target, 1)
            return before

        def send(value):
            os.write(master, (value + "\n").encode())

        try:
            expect("Nome do projeto")
            send("projeto-pty")
            expect("Tipo de projeto")
            expect("Escolha:")
            send("99")
            expect("Escolha uma opção da lista")
            expect("Escolha:")
            send("2" if backend else "1")
            expect(str(root / "projects/projeto-pty"))
            send("")
            if backend:
                expect("Banco de dados")
                options = expect("Escolha:")
                assert b"postgres" in options and b"mongodb" not in options
                send("mongodb")
                expect("Escolha uma opção da lista")
                expect("Escolha:")
                send("2" if postgres else "1")
                if postgres:
                    expect("Localização do banco")
                    send("")
                    expect("Adicionar migrations PostgreSQL versionadas")
                    send("s")
            skipped = expect("Gerar CI GitHub Actions dos projetos")
            if backend:
                for unsupported in [b"Redis", b"Vault", b"Prometheus"]:
                    assert unsupported not in skipped, skipped
            send("")
            expect("Criar estes projetos?")
            send("talvez")
            expect("Responda s ou n")
            send("n" if cancel else "s")
            status = process.wait(timeout=10)
            assert status == (130 if cancel else 0), status
            destination = root / "projects/projeto-pty"
            assert destination.exists() != cancel
            if not cancel:
                assert (
                    destination / ("cmd/api/main.go" if backend else "index.html")
                ).is_file()
                if backend:
                    assert (
                        destination / "internal/infrastructure/database"
                    ).exists() == postgres
                    assert (destination / "scripts/migrate.sh").exists() == postgres
                assert not (destination / "node_modules").exists()
                assert not (destination / ".git").exists()
            assert not any(
                path.name.startswith(".boilerplate-stage-") for path in root.rglob("*")
            )
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
            os.close(master)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    session(binary, cancel=False)
    session(binary, cancel=True)
    session(binary, cancel=False, backend=True)
    session(binary, cancel=False, backend=True, postgres=True)
    print(
        "OK PTY: geração, destino HOME/projects, entradas inválidas, cancelamento e capacidades Go"
    )


if __name__ == "__main__":
    main()
