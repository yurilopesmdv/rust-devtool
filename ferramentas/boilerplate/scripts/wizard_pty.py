#!/usr/bin/env python3
"""Exercise the real interactive wizard in an isolated Linux/WSL PTY."""

import argparse
import errno
import os
import pty
import select
import subprocess
import tempfile
import time
from pathlib import Path


def session(binary, cancel):
    with tempfile.TemporaryDirectory(prefix="boilerplate-pty-") as temporary:
        root = Path(temporary)
        env = os.environ.copy()
        env.update(HOME=str(root), PATH="/nonexistent", NVM_DIR=str(root / "nvm"))
        master, slave = pty.openpty()
        command = [
            str(binary),
            "new",
            "--frontend",
            "vanilla-js",
            "--node-version",
            "24",
            "--module-system",
            "esm",
            "--containers",
            "none",
            "--kubernetes",
            "none",
            "--monitoring",
            "none",
            "--secrets",
            "env",
            "--api-url",
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
            buffer = buffer.split(target, 1)[1]

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
            send("1")
            expect(str(root / "projects/projeto-pty"))
            send("")
            expect("Gerar CI GitHub Actions dos projetos")
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
                assert (destination / "index.html").is_file()
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
    print("OK PTY: geração, destino HOME/projects, entradas inválidas e cancelamento")


if __name__ == "__main__":
    main()
