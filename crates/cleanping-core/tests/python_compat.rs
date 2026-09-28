//! A database written by the earlier Python version (`legacy/python`) must open unchanged.
//! The fixture holds fake data only.

use std::fs;

use cleanping_core::application::app_state::AppState;
use cleanping_core::application::ports::{CredentialRepository, PromptRepository, RunRepository};
use cleanping_core::infrastructure::sqlite_db::Database;
use cleanping_core::infrastructure::sqlite_repositories::*;

fn copy_of_fixture() -> (tempfile::TempDir, Database) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cleanping.db");
    fs::copy(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/python_v0_1.db"),
        &path,
    )
    .unwrap();
    (dir, Database::new(path))
}

#[test]
fn migrating_a_python_database_changes_nothing_and_keeps_every_row() {
    let (_dir, db) = copy_of_fixture();
    db.migrate().unwrap();

    let credentials = SqliteCredentialRepository::new(db.clone()).list().unwrap();
    assert_eq!(credentials.len(), 1);
    assert_eq!(
        (credentials[0].name.as_str(), credentials[0].model.as_str()),
        ("Fixture", "fixture-model")
    );

    let runs = SqliteRunRepository::new(db.clone()).recent(10).unwrap();
    assert_eq!(runs[0].input_text, "rough fixture text");
    assert_eq!(
        runs[0].output_text.as_deref(),
        Some("Polished fixture text.")
    );
    assert_eq!(runs[0].credential_id, credentials[0].id);

    assert_eq!(
        SqlitePromptRepository::new(db.clone())
            .current()
            .unwrap()
            .as_deref(),
        Some("Fixture prompt")
    );

    let state = AppState::new(SqliteStateRepository::new(db));
    assert_eq!(state.selected_credential_id().unwrap(), credentials[0].id);
    let versions = state.load_versions().unwrap();
    assert_eq!(
        (versions.count(), versions.current()),
        (2, "Polished fixture text.")
    );
}
