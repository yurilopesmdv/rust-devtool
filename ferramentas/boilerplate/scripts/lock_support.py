"""Shared process and publication helpers for repository lock maintenance."""

import subprocess


def run(command, cwd, env):
    result = subprocess.run(
        command,
        cwd=cwd,
        env=env,
        text=True,
        capture_output=True,
        check=False,
        timeout=300,
    )
    if result.returncode:
        message = (result.stdout + result.stderr)[-4000:]
        for key, value in env.items():
            if value and any(
                word in key for word in ["PASSWORD", "TOKEN", "SECRET", "URL"]
            ):
                message = message.replace(value, "<redacted>")
        raise RuntimeError(f"{' '.join(command)} falhou em {cwd}:\n{message}")
    return result.stdout


def normalize_project(text, name):
    identity = f'name = "{name}"'
    if text.count(identity) != 1:
        raise ValueError(
            "Lock precisa conter exatamente uma identidade do projeto de manutenção"
        )
    return text.replace(identity, 'name = "{{PROJECT_NAME}}"')


def publish(snapshots, write):
    changed = []
    for path, content in sorted(snapshots.items()):
        if path.exists() and path.read_text(encoding="utf-8") == content:
            continue
        changed.append(path)
    if write:
        for path in changed:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(snapshots[path], encoding="utf-8")
    return changed
