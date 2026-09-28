//! Deleted history must be gone from the database files, not just hidden from queries.

use crate::application::ports::RunRepository;
use crate::domain::models::{Run, RunStatus};
use crate::infrastructure::sqlite_db::Database;
use crate::infrastructure::sqlite_repositories::SqliteRunRepository;

const INPUT: &str = "MARKER-INPUT-TEXT-7f3a91";
const OUTPUT: &str = "MARKER-OUTPUT-TEXT-c204be";

fn database() -> (tempfile::TempDir, Database) {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::new(dir.path().join("cleanping.db"));
    db.migrate().unwrap();
    (dir, db)
}

fn run_with_markers() -> Run {
    Run {
        input_text: INPUT.into(),
        output_text: Some(OUTPUT.into()),
        status: RunStatus::Ok,
        error_message: None,
        duration_ms: 1,
        credential_name: None,
        model: "m".into(),
        prompt_text: "p".into(),
        id: None,
        created_at: None,
        credential_id: None,
    }
}

/// Whether `marker` can be found in the database file or its write-ahead log.
fn readable_in_files(dir: &std::path::Path, marker: &str) -> bool {
    ["cleanping.db", "cleanping.db-wal"].iter().any(|name| {
        std::fs::read(dir.join(name))
            .unwrap_or_default()
            .windows(marker.len())
            .any(|window| window == marker.as_bytes())
    })
}

#[test]
fn the_markers_are_in_the_file_before_anything_is_deleted() {
    let (dir, db) = database();
    SqliteRunRepository::new(db)
        .add(&run_with_markers())
        .unwrap();
    assert!(readable_in_files(dir.path(), INPUT));
    assert!(readable_in_files(dir.path(), OUTPUT));
}

#[test]
fn clearing_the_history_erases_the_text_from_the_files() {
    let (dir, db) = database();
    let repo = SqliteRunRepository::new(db);
    repo.add(&run_with_markers()).unwrap();
    assert_eq!(repo.delete_all().unwrap(), 1);
    assert!(!readable_in_files(dir.path(), INPUT));
    assert!(!readable_in_files(dir.path(), OUTPUT));
}

#[test]
fn purging_old_history_erases_the_text_from_the_files() {
    let (dir, db) = database();
    let repo = SqliteRunRepository::new(db);
    repo.add(&run_with_markers()).unwrap();
    assert_eq!(repo.delete_before("2999-01-01T00:00:00+00:00").unwrap(), 1);
    assert!(!readable_in_files(dir.path(), INPUT));
    assert!(!readable_in_files(dir.path(), OUTPUT));
}

#[test]
fn deleting_leaves_the_rest_of_the_history_intact() {
    let (_dir, db) = database();
    let repo = SqliteRunRepository::new(db);
    repo.add(&run_with_markers()).unwrap();
    repo.add(&Run {
        input_text: "keep me".into(),
        ..run_with_markers()
    })
    .unwrap();
    assert_eq!(repo.delete_before("1999-01-01T00:00:00+00:00").unwrap(), 0);
    assert_eq!(repo.recent(10).unwrap().len(), 2);
}
