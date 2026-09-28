//! Composition root: wires the core's adapters into its use cases.

use std::path::PathBuf;

use cleanping_core::application::app_state::AppState;
use cleanping_core::application::credentials::CredentialService;
use cleanping_core::application::history::HistoryService;
use cleanping_core::application::prompts::PromptService;
use cleanping_core::domain::errors::Result;
use cleanping_core::infrastructure::paths::{database_path, secrets_path};
use cleanping_core::infrastructure::secrets_file::JsonSecretStore;
use cleanping_core::infrastructure::sqlite_db::Database;
use cleanping_core::infrastructure::sqlite_repositories::{
    SqliteCredentialRepository, SqlitePromptRepository, SqliteRunRepository, SqliteStateRepository,
};

pub struct Services {
    pub credentials: CredentialService<SqliteCredentialRepository, JsonSecretStore>,
    pub history: HistoryService<SqliteRunRepository>,
    pub prompts: PromptService<SqlitePromptRepository>,
    pub state: AppState<SqliteStateRepository>,
    pub db: Database,
    secrets: PathBuf,
}

impl Services {
    pub fn open() -> Result<Self> {
        let db = Database::new(database_path()?);
        db.migrate()?;
        let secrets = secrets_path()?;
        Ok(Self {
            credentials: CredentialService::new(
                SqliteCredentialRepository::new(db.clone()),
                JsonSecretStore::new(&secrets),
            ),
            history: HistoryService::new(SqliteRunRepository::new(db.clone())),
            prompts: PromptService::new(SqlitePromptRepository::new(db.clone())),
            state: AppState::new(SqliteStateRepository::new(db.clone())),
            db,
            secrets,
        })
    }

    /// A fresh handle on the secret file (the store keeps no state of its own).
    pub fn secrets(&self) -> JsonSecretStore {
        JsonSecretStore::new(&self.secrets)
    }
}
