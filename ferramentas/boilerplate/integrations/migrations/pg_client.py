#!/usr/bin/env python3
"""Pass PostgreSQL URI credentials through environment, never process arguments."""

import os
import sys
from urllib.parse import parse_qsl, unquote, urlsplit


def connection_env(source, inherited):
    uri = urlsplit(source)
    if (
        uri.scheme not in ["postgres", "postgresql"]
        or not uri.hostname
        or not uri.path.lstrip("/")
        or uri.fragment
    ):
        raise ValueError("URI PostgreSQL inválida")
    if "," in uri.hostname:
        raise ValueError("URI multihost ainda não suportada")
    env = inherited.copy()
    for key in ["PGSERVICE", "PGSERVICEFILE", "PGHOSTADDR", "PGPASSWORD", "PGOPTIONS"]:
        env.pop(key, None)
    env.update(
        PGHOST=uri.hostname,
        PGPORT=str(uri.port or 5432),
        PGDATABASE=unquote(uri.path[1:]),
        PGSSLMODE="require",
        PGCONNECT_TIMEOUT="5",
    )
    if uri.username is not None:
        env["PGUSER"] = unquote(uri.username)
    if uri.password is not None:
        env["PGPASSWORD"] = unquote(uri.password)
    supported = {
        "sslmode": "PGSSLMODE",
        "sslrootcert": "PGSSLROOTCERT",
        "sslcert": "PGSSLCERT",
        "sslkey": "PGSSLKEY",
        "connect_timeout": "PGCONNECT_TIMEOUT",
    }
    seen = set()
    for key, value in parse_qsl(uri.query, keep_blank_values=True):
        if key not in supported or key in seen:
            raise ValueError("Parâmetro de URI não suportado ou repetido")
        seen.add(key)
        env[supported[key]] = value
    return env


def main():
    try:
        env = connection_env(os.environ["BOILERPLATE_MIGRATION_URL"], os.environ)
    except (ValueError, KeyError):
        print(
            "URI de migrations inválida ou opção não suportada; veja docs/MIGRATIONS.md",
            file=sys.stderr,
        )
        return 2
    os.execvpe("psql", ["psql", *sys.argv[1:]], env)


if __name__ == "__main__":
    raise SystemExit(main())
