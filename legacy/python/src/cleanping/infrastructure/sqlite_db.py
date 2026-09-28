"""SQLite database factory and versioned migrations (convention 07)."""

from __future__ import annotations

import os
import sqlite3
from contextlib import contextmanager
from pathlib import Path
from typing import Iterator

from .clock import utc_now

_PRIVATE_FILE_MODE = 0o600
_PRIVATE_DIR_MODE = 0o700

_SCHEMA_V1 = """
CREATE TABLE credentials (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    api_url TEXT NOT NULL,
    model TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE prompts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    body TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TEXT NOT NULL,
    credential_id INTEGER REFERENCES credentials(id) ON DELETE SET NULL,
    credential_name TEXT,
    model TEXT NOT NULL,
    prompt_text TEXT NOT NULL,
    input_text TEXT NOT NULL,
    output_text TEXT,
    status TEXT NOT NULL CHECK (status IN ('ok', 'error')),
    error_message TEXT,
    duration_ms INTEGER NOT NULL
);

CREATE INDEX idx_runs_created_at ON runs (created_at DESC);

CREATE TABLE app_state (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
"""

MIGRATIONS: tuple[tuple[int, str], ...] = ((1, _SCHEMA_V1),)


class Database:
    """Creates short-lived connections; one migration ledger per file."""

    def __init__(self, path: Path):
        self._path = Path(path)

    @contextmanager
    def connect(self) -> Iterator[sqlite3.Connection]:
        self._path.parent.mkdir(parents=True, exist_ok=True, mode=_PRIVATE_DIR_MODE)
        self._harden(self._path.parent, _PRIVATE_DIR_MODE)
        connection = sqlite3.connect(self._path)
        connection.row_factory = sqlite3.Row
        connection.execute("PRAGMA foreign_keys = ON")
        self._harden(self._path, _PRIVATE_FILE_MODE)
        try:
            yield connection
            connection.commit()
        except Exception:
            connection.rollback()
            raise
        finally:
            connection.close()
            for suffix in ("", "-wal", "-shm"):
                self._harden(Path(str(self._path) + suffix), _PRIVATE_FILE_MODE)

    @staticmethod
    def _harden(path: Path, mode: int) -> None:
        """Prompt history is personal data: keep files owner-only (rule 15)."""
        try:
            os.chmod(path, mode)
        except OSError:
            pass

    def migrate(self) -> None:
        with self.connect() as connection:
            connection.execute("PRAGMA journal_mode = WAL")
            connection.execute(
                "CREATE TABLE IF NOT EXISTS schema_migrations ("
                " version INTEGER PRIMARY KEY,"
                " applied_at TEXT NOT NULL"
                ")"
            )
            applied = {
                row["version"]
                for row in connection.execute("SELECT version FROM schema_migrations")
            }
            for version, script in MIGRATIONS:
                if version in applied:
                    continue
                connection.executescript(script)
                connection.execute(
                    "INSERT INTO schema_migrations(version, applied_at) VALUES (?, ?)",
                    (version, utc_now()),
                )
