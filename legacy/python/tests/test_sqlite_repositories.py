"""SQLite adapters: migrations, credential/run/prompt/state behavior."""

import sqlite3
import stat

import pytest

from cleanping.domain.errors import NotFoundError
from cleanping.domain.models import CredentialInput, Run
from cleanping.infrastructure.sqlite_db import Database
from cleanping.infrastructure.sqlite_repositories import (
    SqliteCredentialRepository,
    SqlitePromptRepository,
    SqliteRunRepository,
    SqliteStateRepository,
)


@pytest.fixture
def db(tmp_path):
    database = Database(tmp_path / "cleanping.db")
    database.migrate()
    return database


def _draft(name="OpenAI", url="https://api.openai.com/v1/chat/completions", model="gpt-4o-mini"):
    return CredentialInput(name=name, api_url=url, model=model, api_key="sk-x")


def _run(**overrides):
    values = {
        "input_text": "pleas fix this",
        "output_text": "please fix this",
        "status": "ok",
        "error_message": None,
        "duration_ms": 12,
        "credential_name": "OpenAI",
        "model": "gpt-4o-mini",
        "prompt_text": "Fix the text.",
    }
    values.update(overrides)
    return Run(**values)


class TestMigrations:
    def test_migrate_is_idempotent(self, tmp_path):
        database = Database(tmp_path / "db.sqlite")
        database.migrate()
        database.migrate()
        with database.connect() as connection:
            versions = [
                row["version"]
                for row in connection.execute("SELECT version FROM schema_migrations")
            ]
        assert versions == [1]

    def test_database_file_is_private(self, tmp_path):
        path = tmp_path / "db.sqlite"
        database = Database(path)
        database.migrate()
        assert stat.S_IMODE(path.stat().st_mode) == 0o600


class TestCredentialRepository:
    def test_insert_then_update_by_name(self, db):
        repo = SqliteCredentialRepository(db)
        created = repo.upsert(_draft())
        assert created.id is not None
        updated = repo.upsert(_draft(model="gpt-4o"))
        assert updated.id == created.id
        assert repo.list() == [updated]

    def test_list_is_sorted_by_name(self, db):
        repo = SqliteCredentialRepository(db)
        repo.upsert(_draft(name="zeta"))
        repo.upsert(_draft(name="Alpha"))
        assert [item.name for item in repo.list()] == ["Alpha", "zeta"]

    def test_get_unknown_id_raises(self, db):
        with pytest.raises(NotFoundError):
            SqliteCredentialRepository(db).get(999)

    def test_delete_removes_row(self, db):
        repo = SqliteCredentialRepository(db)
        created = repo.upsert(_draft())
        repo.delete(created.id)
        assert repo.list() == []
        with pytest.raises(NotFoundError):
            repo.get(created.id)


class TestRunRepository:
    def test_add_stamps_id_and_time(self, db):
        repo = SqliteRunRepository(db)
        stored = repo.add(_run())
        assert stored.id is not None
        assert stored.created_at is not None
        assert repo.recent()[0].input_text == "pleas fix this"

    def test_recent_orders_newest_first_and_limits(self, db):
        repo = SqliteRunRepository(db)
        for index in range(3):
            repo.add(_run(input_text=f"draft {index}"))
        assert [item.input_text for item in repo.recent(limit=2)] == [
            "draft 2",
            "draft 1",
        ]

    def test_run_survives_credential_delete_and_keeps_name(self, db):
        credentials = SqliteCredentialRepository(db)
        credential = credentials.upsert(_draft())
        runs = SqliteRunRepository(db)
        runs.add(_run(credential_id=credential.id))
        credentials.delete(credential.id)
        stored = runs.recent()[0]
        assert stored.credential_id is None
        assert stored.credential_name == "OpenAI"

    def test_error_run_keeps_message(self, db):
        repo = SqliteRunRepository(db)
        repo.add(
            _run(status="error", output_text=None, error_message="API returned HTTP 401.")
        )
        stored = repo.recent()[0]
        assert stored.status == "error"
        assert stored.error_message == "API returned HTTP 401."

    def test_status_check_rejects_unknown_values(self, db):
        repo = SqliteRunRepository(db)
        with pytest.raises(sqlite3.IntegrityError):
            repo.add(_run(status="weird"))


class TestPromptRepository:
    def test_current_is_none_before_first_save(self, db):
        assert SqlitePromptRepository(db).current() is None

    def test_save_appends_and_current_is_latest(self, db):
        repo = SqlitePromptRepository(db)
        repo.save("first prompt")
        repo.save("second prompt")
        assert repo.current() == "second prompt"


class TestStateRepository:
    def test_get_missing_key_returns_none(self, db):
        assert SqliteStateRepository(db).get("draft") is None

    def test_set_inserts_then_updates(self, db):
        repo = SqliteStateRepository(db)
        repo.set("draft", "hello")
        repo.set("draft", "hello world")
        assert repo.get("draft") == "hello world"
