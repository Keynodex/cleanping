"""SQLite adapters for the repository ports. No business rules here."""

from __future__ import annotations

import sqlite3

from ..domain.errors import NotFoundError
from ..domain.models import Credential, CredentialInput, Run
from .clock import utc_now
from .sqlite_db import Database

_CREDENTIAL_COLUMNS = "id, name, api_url, model"


class SqliteCredentialRepository:
    def __init__(self, db: Database):
        self._db = db

    def list(self) -> list[Credential]:
        with self._db.connect() as connection:
            rows = connection.execute(
                f"SELECT {_CREDENTIAL_COLUMNS} FROM credentials ORDER BY name COLLATE NOCASE"
            ).fetchall()
        return [_to_credential(row) for row in rows]

    def get(self, credential_id: int) -> Credential:
        with self._db.connect() as connection:
            row = connection.execute(
                f"SELECT {_CREDENTIAL_COLUMNS} FROM credentials WHERE id = ?",
                (credential_id,),
            ).fetchone()
        if row is None:
            raise NotFoundError(f"Credential {credential_id} no longer exists.")
        return _to_credential(row)

    def upsert(self, draft: CredentialInput) -> Credential:
        now = utc_now()
        with self._db.connect() as connection:
            row = connection.execute(
                "SELECT id FROM credentials WHERE name = ?", (draft.name,)
            ).fetchone()
            if row is None:
                cursor = connection.execute(
                    "INSERT INTO credentials (name, api_url, model, created_at, updated_at)"
                    " VALUES (?, ?, ?, ?, ?)",
                    (draft.name, draft.api_url, draft.model, now, now),
                )
                credential_id = int(cursor.lastrowid)
            else:
                credential_id = int(row["id"])
                connection.execute(
                    "UPDATE credentials SET api_url = ?, model = ?, updated_at = ?"
                    " WHERE id = ?",
                    (draft.api_url, draft.model, now, credential_id),
                )
        return Credential(
            id=credential_id,
            name=draft.name,
            api_url=draft.api_url,
            model=draft.model,
        )

    def delete(self, credential_id: int) -> None:
        with self._db.connect() as connection:
            connection.execute("DELETE FROM credentials WHERE id = ?", (credential_id,))


class SqliteRunRepository:
    def __init__(self, db: Database):
        self._db = db

    def add(self, run: Run) -> Run:
        created_at = utc_now()
        with self._db.connect() as connection:
            cursor = connection.execute(
                "INSERT INTO runs (created_at, credential_id, credential_name, model,"
                " prompt_text, input_text, output_text, status, error_message, duration_ms)"
                " VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                (
                    created_at,
                    run.credential_id,
                    run.credential_name,
                    run.model,
                    run.prompt_text,
                    run.input_text,
                    run.output_text,
                    run.status,
                    run.error_message,
                    run.duration_ms,
                ),
            )
            run_id = int(cursor.lastrowid)
        return Run(
            input_text=run.input_text,
            output_text=run.output_text,
            status=run.status,
            error_message=run.error_message,
            duration_ms=run.duration_ms,
            credential_name=run.credential_name,
            model=run.model,
            prompt_text=run.prompt_text,
            id=run_id,
            created_at=created_at,
            credential_id=run.credential_id,
        )

    def recent(self, limit: int = 50) -> list[Run]:
        with self._db.connect() as connection:
            rows = connection.execute(
                "SELECT * FROM runs ORDER BY created_at DESC, id DESC LIMIT ?",
                (limit,),
            ).fetchall()
        return [_to_run(row) for row in rows]


class SqlitePromptRepository:
    def __init__(self, db: Database):
        self._db = db

    def current(self) -> str | None:
        with self._db.connect() as connection:
            row = connection.execute(
                "SELECT body FROM prompts ORDER BY id DESC LIMIT 1"
            ).fetchone()
        return row["body"] if row else None

    def save(self, body: str) -> None:
        with self._db.connect() as connection:
            connection.execute(
                "INSERT INTO prompts (body, created_at) VALUES (?, ?)", (body, utc_now())
            )


class SqliteStateRepository:
    def __init__(self, db: Database):
        self._db = db

    def get(self, key: str) -> str | None:
        with self._db.connect() as connection:
            row = connection.execute(
                "SELECT value FROM app_state WHERE key = ?", (key,)
            ).fetchone()
        return row["value"] if row else None

    def set(self, key: str, value: str) -> None:
        with self._db.connect() as connection:
            connection.execute(
                "INSERT INTO app_state (key, value, updated_at) VALUES (?, ?, ?)"
                " ON CONFLICT(key) DO UPDATE SET value = excluded.value,"
                " updated_at = excluded.updated_at",
                (key, value, utc_now()),
            )


def _to_credential(row: sqlite3.Row) -> Credential:
    return Credential(
        id=row["id"], name=row["name"], api_url=row["api_url"], model=row["model"]
    )


def _to_run(row: sqlite3.Row) -> Run:
    return Run(
        input_text=row["input_text"],
        output_text=row["output_text"],
        status=row["status"],
        error_message=row["error_message"],
        duration_ms=row["duration_ms"],
        credential_name=row["credential_name"],
        model=row["model"],
        prompt_text=row["prompt_text"],
        id=row["id"],
        created_at=row["created_at"],
        credential_id=row["credential_id"],
    )
