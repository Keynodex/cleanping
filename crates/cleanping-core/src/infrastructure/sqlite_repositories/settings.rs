//! The saved system prompt (append-only) and small key/value state.

use rusqlite::{params, OptionalExtension};

use crate::application::ports::{PromptRepository, StateRepository};
use crate::domain::errors::Result;
use crate::infrastructure::clock::utc_now;

repository!(SqlitePromptRepository, SqliteStateRepository);

impl PromptRepository for SqlitePromptRepository {
    fn current(&self) -> Result<Option<String>> {
        self.db.with(|c| {
            c.query_row(
                "SELECT body FROM prompts ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()
        })
    }

    fn save(&self, body: &str) -> Result<()> {
        self.db.with(|c| {
            c.execute(
                "INSERT INTO prompts (body, created_at) VALUES (?1, ?2)",
                params![body, utc_now()],
            )
            .map(|_| ())
        })
    }
}

impl StateRepository for SqliteStateRepository {
    fn get(&self, key: &str) -> Result<Option<String>> {
        self.db.with(|c| {
            c.query_row("SELECT value FROM app_state WHERE key = ?1", [key], |row| {
                row.get(0)
            })
            .optional()
        })
    }

    fn set(&self, key: &str, value: &str) -> Result<()> {
        self.db.with(|c| {
            c.execute(
                "INSERT INTO app_state (key, value, updated_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                params![key, value, utc_now()],
            )
            .map(|_| ())
        })
    }
}
