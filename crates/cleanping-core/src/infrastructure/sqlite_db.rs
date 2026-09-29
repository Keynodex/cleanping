//! SQLite database factory and versioned migrations. The schema is byte-compatible with
//! the Python release, so an existing `cleanping.db` opens unchanged.

use std::path::PathBuf;

use rusqlite::{params, Connection};

use super::clock::utc_now;
use super::private_fs::{ensure_private_dir, tighten_file};
use crate::domain::errors::{CleanpingError, Result};

const SCHEMA_V1: &str = "
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
";

const MIGRATIONS: &[(i64, &str)] = &[(1, SCHEMA_V1)];

/// Handle on the SQLite database file. Cheap to clone: each call opens a short-lived
/// connection with `foreign_keys` and `secure_delete` on, and on Unix keeps the file and its
/// folder owner-only.
#[derive(Clone, Debug)]
pub struct Database {
    path: PathBuf,
}

fn storage(error: impl std::fmt::Display) -> CleanpingError {
    CleanpingError::Storage(format!("Database error: {error}."))
}

impl Database {
    /// Point at the database file at `path`; nothing is opened until first use. Run
    /// [`migrate`](Self::migrate) before using the repositories.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    fn side_files(&self) -> [PathBuf; 3] {
        let with = |suffix: &str| {
            let mut name = self.path.clone().into_os_string();
            name.push(suffix);
            PathBuf::from(name)
        };
        [self.path.clone(), with("-wal"), with("-shm")]
    }

    /// Open a short-lived connection, run `f`, then re-harden the file modes.
    pub fn with<T>(&self, f: impl FnOnce(&Connection) -> rusqlite::Result<T>) -> Result<T> {
        if let Some(directory) = self.path.parent().filter(|d| !d.as_os_str().is_empty()) {
            ensure_private_dir(directory).map_err(storage)?;
        }
        let connection = Connection::open(&self.path).map_err(storage)?;
        tighten_file(&self.path);
        // secure_delete overwrites deleted content with zeros, so removed history and keys
        // cannot be read back from the file's free pages.
        connection
            .execute_batch("PRAGMA foreign_keys = ON; PRAGMA secure_delete = ON")
            .map_err(storage)?;
        let outcome = f(&connection);
        drop(connection);
        for file in self.side_files() {
            tighten_file(&file);
        }
        outcome.map_err(storage)
    }

    /// Create the file and its folder if needed, switch to WAL mode and apply any missing schema
    /// migrations. Safe to run on every launch, including several launches at once.
    pub fn migrate(&self) -> Result<()> {
        self.with(|connection| {
            ensure_wal(connection)?;
            for (version, script) in MIGRATIONS {
                // IMMEDIATE takes the write lock up front: a second launch waits here and then
                // finds the migration already applied instead of colliding with it.
                connection.execute_batch("BEGIN IMMEDIATE")?;
                match apply_migration(connection, *version, script) {
                    Ok(()) => connection.execute_batch("COMMIT")?,
                    Err(error) => {
                        let _ = connection.execute_batch("ROLLBACK");
                        return Err(error);
                    }
                }
            }
            Ok(())
        })
    }
}

/// Switch to WAL mode once. SQLite refuses that switch outright (no waiting) while another
/// connection is active, so look first and retry briefly: a parallel launch may be doing it.
fn ensure_wal(connection: &Connection) -> rusqlite::Result<()> {
    const ATTEMPTS: u32 = 100;
    for attempt in 1..=ATTEMPTS {
        let mode: String = connection.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
        if mode.eq_ignore_ascii_case("wal") {
            return Ok(());
        }
        match connection.query_row("PRAGMA journal_mode = WAL", [], |row| {
            row.get::<_, String>(0)
        }) {
            Ok(_) => return Ok(()),
            Err(error) if is_busy(&error) && attempt < ATTEMPTS => {
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn is_busy(error: &rusqlite::Error) -> bool {
    matches!(
        error,
        rusqlite::Error::SqliteFailure(failure, _)
            if matches!(failure.code, rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked)
    )
}

fn apply_migration(connection: &Connection, version: i64, script: &str) -> rusqlite::Result<()> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
             version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)",
    )?;
    let applied: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
        [version],
        |row| row.get(0),
    )?;
    if applied {
        return Ok(());
    }
    connection.execute_batch(script)?;
    connection.execute(
        "INSERT INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
        params![version, utc_now()],
    )?;
    Ok(())
}
