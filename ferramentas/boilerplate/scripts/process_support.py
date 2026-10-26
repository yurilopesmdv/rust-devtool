"""Bounded Linux subprocesses; cancellation terminates the complete process group."""

import os
import signal
import subprocess


def capture(command, cwd, env, timeout=900):
    process = subprocess.Popen(
        command,
        cwd=cwd,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        stdin=subprocess.DEVNULL,
        start_new_session=True,
    )
    try:
        stdout, stderr = process.communicate(timeout=timeout)
    except (subprocess.TimeoutExpired, KeyboardInterrupt) as error:
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            process.communicate(timeout=0.5)
        except (subprocess.TimeoutExpired, KeyboardInterrupt):
            pass
        finally:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.communicate()
        reason = "interrompido" if isinstance(error, KeyboardInterrupt) else "timeout"
        raise RuntimeError(
            f"{command[0]}: {reason} em {cwd}; arquivos preservados; repita a etapa após corrigir a causa"
        ) from None
    return subprocess.CompletedProcess(command, process.returncode, stdout, stderr)
