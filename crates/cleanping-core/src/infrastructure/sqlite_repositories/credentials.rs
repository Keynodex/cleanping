//! Saved provider endpoints. The API key itself lives in the secret store, never here.

use rusqlite::{params, OptionalExtension, Row};

use crate::application::ports::CredentialRepository;
use crate::domain::errors::{CleanpingError, Result};
use crate::domain::models::{Credential, CredentialInput};
use crate::infrastructure::clock::utc_now;

repository!(SqliteCredentialRepository);

fn to_credential(row: &Row) -> rusqlite::Result<Credential> {
    Ok(Credential {
        id: Some(row.get("id")?),
        name: row.get("name")?,
        api_url: row.get("api_url")?,
        model: row.get("model")?,
    })
}

impl CredentialRepository for SqliteCredentialRepository {
    fn list(&self) -> Result<Vec<Credential>> {
        self.db.with(|c| {
            c.prepare(
                "SELECT id, name, api_url, model FROM credentials ORDER BY name COLLATE NOCASE",
            )?
            .query_map([], to_credential)?
            .collect()
        })
    }

    fn get(&self, id: i64) -> Result<Credential> {
        self.db
            .with(|c| {
                c.query_row(
                    "SELECT id, name, api_url, model FROM credentials WHERE id = ?1",
                    [id],
                    to_credential,
                )
                .optional()
            })?
            .ok_or_else(|| CleanpingError::NotFound(format!("Credential {id} no longer exists.")))
    }

    fn upsert(&self, draft: &CredentialInput) -> Result<Credential> {
        let now = utc_now();
        let id = self.db.with(|c| {
            c.query_row(
                "INSERT INTO credentials (name, api_url, model, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?4)
                 ON CONFLICT(name) DO UPDATE SET
                     api_url = excluded.api_url, model = excluded.model, updated_at = excluded.updated_at
                 RETURNING id",
                params![draft.name, draft.api_url, draft.model, now],
                |row| row.get::<_, i64>(0),
            )
        })?;
        Ok(Credential {
            id: Some(id),
            name: draft.name.clone(),
            api_url: draft.api_url.clone(),
            model: draft.model.clone(),
        })
    }

    fn delete(&self, id: i64) -> Result<()> {
        self.db.with(|c| {
            c.execute("DELETE FROM credentials WHERE id = ?1", [id])
                .map(|_| ())
        })
    }
}
