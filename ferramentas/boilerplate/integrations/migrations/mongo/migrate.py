#!/usr/bin/env python3
"""Standalone migrations using the application's existing PyMongo driver."""

import hashlib
import json
import os
import re
import sys
import time
import uuid
from datetime import UTC, datetime
from pathlib import Path

from pymongo import MongoClient
from pymongo.errors import DuplicateKeyError, PyMongoError

ROOT = Path(__file__).resolve().parents[1]
OWNER = uuid.uuid4().hex


def now():
    return datetime.now(UTC)


def configuration():
    prefix = (
        "MIGRATION_DATABASE_URL"
        if os.environ.get("MIGRATION_DATABASE_URL") or os.environ.get("MIGRATION_DATABASE_URL_FILE")
        else "DATABASE_URL"
    )
    direct = os.environ.get(prefix)
    file = os.environ.get(prefix + "_FILE")
    if direct and file:
        raise ValueError("Configuração ambígua")
    url = direct or (Path(file).read_text().rstrip("\r\n") if file else None)
    if not url or not re.match(r"^mongodb(?:\+srv)?://", url):
        raise ValueError("URI Mongo obrigatória")
    return url


def load():
    files = []
    ids = set()
    for path in sorted((ROOT / "migrations/mongo").glob("*.json")):
        if (
            path.is_symlink()
            or not path.is_file()
            or not re.fullmatch(r"[0-9]{14}_[a-z0-9_-]+\.json", path.name)
        ):
            raise ValueError("Arquivo inválido")
        version = path.name[:14]
        if version in ids:
            raise ValueError("Versão duplicada")
        ids.add(version)
        raw = path.read_bytes()
        if len(raw) > 1048576 or raw.startswith(b"\xef\xbb\xbf"):
            raise ValueError("Arquivo inválido")
        spec = json.loads(raw.decode("utf-8"))
        if not isinstance(spec, dict):
            raise TypeError("Objeto obrigatório")
        allowed = (
            {"operation"}
            if spec.get("operation") == "bootstrap_examples"
            else {"operation", "source", "target", "batch_size"}
        )
        if set(spec) - allowed:
            raise ValueError("Opção desconhecida")
        if spec.get("operation") == "copy_field":
            if not all(
                isinstance(spec.get(field), str)
                and re.fullmatch(r"[a-z][a-z0-9_]{0,63}", spec[field])
                for field in ["source", "target"]
            ):
                raise ValueError("Campo inválido")
            batch = spec.get("batch_size")
            if (
                spec["source"] == spec["target"]
                or type(batch) not in (int, float)
                or not 1 <= batch <= 1000
                or batch % 1 != 0
            ):
                raise ValueError("Backfill inválido")
            spec["batch_size"] = int(batch)
        elif spec.get("operation") != "bootstrap_examples":
            raise ValueError("Operação desconhecida")
        files.append({"id": version, "checksum": hashlib.sha256(raw).hexdigest(), "spec": spec})
    return files


def check_history(ledger, files):
    records = list(ledger.find({}))
    expected = {file["id"]: file["checksum"] for file in files}
    if any(expected.get(row["_id"]) != row.get("checksum") for row in records):
        raise ValueError("Histórico divergente")
    return records


def run(client, files, args):
    database = client.get_default_database()
    database.command("ping")
    locks = database["_boilerplate_migration_lock"]
    ledger = database["_boilerplate_migrations"]
    examples = database["examples"]
    if args and args[0] == "unlock":
        if (
            len(args) != 3
            or args[2] != "--confirm-stopped"
            or not re.fullmatch(r"[a-zA-Z0-9-]{1,128}", args[1])
        ):
            raise ValueError("Confirmação obrigatória")
        if locks.delete_one({"_id": "runner", "owner": args[1]}).deleted_count != 1:
            raise ValueError("Reserva não corresponde")
        print(json.dumps({"unlocked": True}))
        return
    if len(args) != 1 or args[0] not in ["status", "up"] or not files:
        raise ValueError("Uso inválido")
    records = check_history(ledger, files)
    if args[0] == "status":
        reservation = locks.find_one({"_id": "runner"})
        states = {row["_id"]: row for row in records}
        print(
            json.dumps(
                {
                    "reserved_by": reservation.get("owner") if reservation else None,
                    "migrations": [
                        {
                            "version": file["id"],
                            "status": states.get(file["id"], {}).get("status", "pending"),
                            "checkpoint": states.get(file["id"], {}).get("checkpoint", 0),
                        }
                        for file in files
                    ],
                }
            )
        )
        return
    acquired = False
    try:
        deadline = time.monotonic() + 30
        while not acquired:
            if time.monotonic() >= deadline:
                raise ValueError("Reserva indisponível")
            try:
                locks.insert_one({"_id": "runner", "owner": OWNER, "started_at": now()})
                acquired = True
            except DuplicateKeyError:
                time.sleep(0.05)
        current = check_history(ledger, files)
        highest = max(
            (row["_id"] for row in current if row.get("status") == "applied"),
            default="",
        )
        for file in files:
            existing = ledger.find_one({"_id": file["id"]})
            if existing and existing.get("status") == "applied":
                continue
            if file["id"] < highest:
                raise ValueError("Migration fora de ordem")
            ledger.update_one(
                {"_id": file["id"]},
                {
                    "$setOnInsert": {
                        "checksum": file["checksum"],
                        "checkpoint": 0,
                        "created_at": now(),
                    },
                    "$set": {"status": "running", "owner": OWNER, "updated_at": now()},
                },
                upsert=True,
            )
            try:
                spec = file["spec"]
                if spec["operation"] == "bootstrap_examples":
                    examples.create_index("message")
                    examples.update_one(
                        {"_id": "default"},
                        {"$setOnInsert": {"message": "API funcionando"}},
                        upsert=True,
                    )
                else:
                    while True:
                        if not locks.find_one({"_id": "runner", "owner": OWNER}):
                            raise ValueError("Reserva perdida")
                        docs = list(
                            examples.find({spec["target"]: {"$exists": False}})
                            .sort("_id", 1)
                            .limit(spec["batch_size"])
                        )
                        if not docs:
                            break
                        for doc in docs:
                            if not isinstance(doc.get(spec["source"]), str):
                                raise TypeError("Fonte do backfill precisa ser string")
                            changed = examples.update_one(
                                {"_id": doc["_id"], spec["target"]: {"$exists": False}},
                                {"$set": {spec["target"]: doc[spec["source"]]}},
                            )
                            ledger.update_one(
                                {"_id": file["id"], "owner": OWNER},
                                {
                                    "$inc": {"checkpoint": changed.modified_count},
                                    "$set": {"updated_at": now()},
                                },
                            )
                if not locks.find_one({"_id": "runner", "owner": OWNER}):
                    raise ValueError("Reserva perdida")
                ledger.update_one(
                    {"_id": file["id"], "owner": OWNER},
                    {
                        "$set": {"status": "applied", "updated_at": now()},
                        "$unset": {"error_code": ""},
                    },
                )
            except Exception:
                ledger.update_one(
                    {"_id": file["id"], "owner": OWNER},
                    {
                        "$set": {
                            "status": "failed",
                            "error_code": "execution_failed",
                            "updated_at": now(),
                        }
                    },
                )
                raise
        print(json.dumps({"applied": True}))
    finally:
        if acquired:
            locks.delete_one({"_id": "runner", "owner": OWNER})


def main():
    client = None
    try:
        args = sys.argv[1:]
        files = [] if args and args[0] == "unlock" else load()
        client = MongoClient(
            configuration(),
            serverSelectionTimeoutMS=5000,
            connectTimeoutMS=5000,
            socketTimeoutMS=5000,
        )
        run(client, files, args)
    except KeyboardInterrupt:
        print(
            "Migration Mongo cancelada; confira status/reserva antes de retomar",
            file=sys.stderr,
        )
        return 130
    except (PyMongoError, ValueError, TypeError, OSError, KeyError):
        print(
            "Migration Mongo falhou; verifique configuração, reserva, checksum e dados. "
            "Consulte docs/MIGRATIONS.md.",
            file=sys.stderr,
        )
        return 1
    finally:
        if client is not None:
            client.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
