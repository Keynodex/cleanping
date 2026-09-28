//! The rewrite history: every input, output and system prompt, kept locally.

use rusqlite::{params, Row};

use crate::application::ports::RunRepository;
use crate::domain::errors::Result;
use crate::domain::models::{Run, RunStatus};
use crate::infrastructure::clock::utc_now;

repository!(SqliteRunRepository);

fn to_run(row: &Row) -> rusqlite::Result<Run> {
    let status: String = row.get("status")?;
    let status = RunStatus::parse(&status).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            "unknown run status".into(),
        )
    })?;
    Ok(Run {
        input_text: row.get("input_text")?,
        output_text: row.get("output_text")?,
        status,
        error_message: row.get("error_message")?,
        duration_ms: row.get("duration_ms")?,
        credential_name: row.get("credential_name")?,
        model: row.get("model")?,
        prompt_text: row.get("prompt_text")?,
        id: Some(row.get("id")?),
        created_at: Some(row.get("created_at")?),
        credential_id: row.get("credential_id")?,
    })
}

/// Rewrite the database file so nothing deleted survives in old pages (this also clears
/// leftovers from before secure deletion was on).
fn rebuild_file(connection: &rusqlite::Connection) -> rusqlite::Result<()> {
    connection.execute_batch("VACUUM")
}

impl RunRepository for SqliteRunRepository {
    fn add(&self, run: &Run) -> Result<Run> {
        let created_at = utc_now();
        let id = self.db.with(|c| {
            c.execute(
                "INSERT INTO runs (created_at, credential_id, credential_name, model, prompt_text,
                                   input_text, output_text, status, error_message, duration_ms)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    created_at,
                    run.credential_id,
                    run.credential_name,
                    run.model,
                    run.prompt_text,
                    run.input_text,
                    run.output_text,
                    run.status.as_str(),
                    run.error_message,
                    run.duration_ms
                ],
            )?;
            Ok(c.last_insert_rowid())
        })?;
        Ok(Run {
            id: Some(id),
            created_at: Some(created_at),
            ..run.clone()
        })
    }

    fn recent(&self, limit: usize) -> Result<Vec<Run>> {
        let limit = i64::try_from(limit).unwrap_or(i64::MAX);
        self.db.with(|c| {
            c.prepare("SELECT * FROM runs ORDER BY created_at DESC, id DESC LIMIT ?1")?
                .query_map([limit], to_run)?
                .collect()
        })
    }

    fn delete_all(&self) -> Result<usize> {
        self.db.with(|c| {
            let removed = c.execute("DELETE FROM runs", [])?;
            rebuild_file(c)?;
            Ok(removed)
        })
    }

    fn delete_before(&self, cutoff: &str) -> Result<usize> {
        self.db.with(|c| {
            let removed = c.execute("DELETE FROM runs WHERE created_at < ?1", [cutoff])?;
            rebuild_file(c)?;
            Ok(removed)
        })
    }
}
