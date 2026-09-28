use super::*;
use crate::application::ports::{
    CredentialRepository, PromptRepository, RunRepository, StateRepository,
};
use crate::domain::errors::CleanpingError;
use crate::domain::models::RunStatus;
use crate::domain::models::{CredentialInput, Run};
use crate::infrastructure::sqlite_db::Database;
use std::os::unix::fs::PermissionsExt;

fn database() -> (tempfile::TempDir, Database) {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::new(dir.path().join("cleanping.db"));
    db.migrate().unwrap();
    (dir, db)
}

fn draft(name: &str, model: &str) -> CredentialInput {
    CredentialInput {
        name: name.into(),
        api_url: "https://api.openai.com/v1/chat/completions".into(),
        model: model.into(),
        api_key: "sk-x".into(),
    }
}

fn run(input: &str) -> Run {
    Run {
        input_text: input.into(),
        output_text: Some("please fix this".into()),
        status: RunStatus::Ok,
        error_message: None,
        duration_ms: 12,
        credential_name: Some("OpenAI".into()),
        model: "gpt-4o-mini".into(),
        prompt_text: "Fix the text.".into(),
        id: None,
        created_at: None,
        credential_id: None,
    }
}

#[test]
fn migrate_is_idempotent_and_the_file_is_private() {
    let (dir, db) = database();
    db.migrate().unwrap();
    let versions: Vec<i64> = db
        .with(|c| {
            c.prepare("SELECT version FROM schema_migrations")?
                .query_map([], |r| r.get(0))?
                .collect()
        })
        .unwrap();
    assert_eq!(versions, [1]);
    let mode = std::fs::metadata(dir.path().join("cleanping.db"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
}

#[test]
fn credential_insert_then_update_by_name() {
    let (_dir, db) = database();
    let repo = SqliteCredentialRepository::new(db);
    let created = repo.upsert(&draft("OpenAI", "gpt-4o-mini")).unwrap();
    let updated = repo.upsert(&draft("OpenAI", "gpt-4o")).unwrap();
    assert!(created.id.is_some());
    assert_eq!(updated.id, created.id);
    assert_eq!(repo.list().unwrap(), [updated]);
}

#[test]
fn credentials_list_sorted_by_name_ignoring_case() {
    let (_dir, db) = database();
    let repo = SqliteCredentialRepository::new(db);
    repo.upsert(&draft("zeta", "m")).unwrap();
    repo.upsert(&draft("Alpha", "m")).unwrap();
    let names: Vec<String> = repo.list().unwrap().into_iter().map(|c| c.name).collect();
    assert_eq!(names, ["Alpha", "zeta"]);
}

#[test]
fn credential_get_unknown_id_and_delete() {
    let (_dir, db) = database();
    let repo = SqliteCredentialRepository::new(db);
    assert!(matches!(repo.get(999), Err(CleanpingError::NotFound(_))));
    let created = repo.upsert(&draft("OpenAI", "m")).unwrap();
    repo.delete(created.id.unwrap()).unwrap();
    assert!(repo.list().unwrap().is_empty());
    assert!(matches!(
        repo.get(created.id.unwrap()),
        Err(CleanpingError::NotFound(_))
    ));
}

#[test]
fn run_add_stamps_id_and_time() {
    let (_dir, db) = database();
    let repo = SqliteRunRepository::new(db);
    let stored = repo.add(&run("pleas fix this")).unwrap();
    assert!(stored.id.is_some() && stored.created_at.is_some());
    assert_eq!(repo.recent(50).unwrap()[0].input_text, "pleas fix this");
}

#[test]
fn recent_orders_newest_first_and_limits() {
    let (_dir, db) = database();
    let repo = SqliteRunRepository::new(db);
    for n in 0..3 {
        repo.add(&run(&format!("draft {n}"))).unwrap();
    }
    let inputs: Vec<String> = repo
        .recent(2)
        .unwrap()
        .into_iter()
        .map(|r| r.input_text)
        .collect();
    assert_eq!(inputs, ["draft 2", "draft 1"]);
}

fn run_stamped(db: &Database, stamp: &str, input: &str) {
    db.with(|c| {
        c.execute(
            "INSERT INTO runs (created_at, model, prompt_text, input_text, status, duration_ms)
             VALUES (?1, 'm', 'p', ?2, 'ok', 1)",
            [stamp, input],
        )
    })
    .unwrap();
}

#[test]
fn delete_all_empties_the_history_and_counts() {
    let (_dir, db) = database();
    let repo = SqliteRunRepository::new(db);
    repo.add(&run("a")).unwrap();
    repo.add(&run("b")).unwrap();
    assert_eq!(repo.delete_all().unwrap(), 2);
    assert!(repo.recent(50).unwrap().is_empty());
    assert_eq!(repo.delete_all().unwrap(), 0);
}

#[test]
fn delete_before_keeps_runs_at_or_after_the_cutoff() {
    let (_dir, db) = database();
    run_stamped(&db, "2026-01-01T00:00:00+00:00", "old");
    run_stamped(&db, "2026-04-01T00:00:00+00:00", "edge");
    run_stamped(&db, "2026-06-01T00:00:00+00:00", "new");
    let repo = SqliteRunRepository::new(db);
    assert_eq!(repo.delete_before("2026-04-01T00:00:00+00:00").unwrap(), 1);
    let inputs: Vec<String> = repo
        .recent(50)
        .unwrap()
        .into_iter()
        .map(|r| r.input_text)
        .collect();
    assert_eq!(inputs, ["new", "edge"]);
}

#[test]
fn run_survives_credential_delete_and_keeps_the_name() {
    let (_dir, db) = database();
    let credentials = SqliteCredentialRepository::new(db.clone());
    let credential = credentials.upsert(&draft("OpenAI", "m")).unwrap();
    let runs = SqliteRunRepository::new(db);
    runs.add(&Run {
        credential_id: credential.id,
        ..run("x")
    })
    .unwrap();
    credentials.delete(credential.id.unwrap()).unwrap();
    let stored = &runs.recent(50).unwrap()[0];
    assert_eq!(
        (stored.credential_id, stored.credential_name.as_deref()),
        (None, Some("OpenAI"))
    );
}

#[test]
fn error_run_keeps_its_message() {
    let (_dir, db) = database();
    let repo = SqliteRunRepository::new(db);
    let failed = Run {
        status: RunStatus::Error,
        output_text: None,
        error_message: Some("API returned HTTP 401.".into()),
        ..run("x")
    };
    repo.add(&failed).unwrap();
    let stored = &repo.recent(50).unwrap()[0];
    assert_eq!(
        (stored.status, stored.error_message.as_deref()),
        (RunStatus::Error, Some("API returned HTTP 401."))
    );
}

#[test]
fn the_database_itself_rejects_unknown_run_statuses() {
    let (_dir, db) = database();
    let result = db.with(|c| {
        c.execute(
            "INSERT INTO runs (created_at, model, prompt_text, input_text, status, duration_ms)
             VALUES ('t', 'm', 'p', 'i', 'weird', 1)",
            [],
        )
    });
    assert!(result.is_err());
}

#[test]
fn prompts_are_append_only_and_current_is_the_latest() {
    let (_dir, db) = database();
    let repo = SqlitePromptRepository::new(db);
    assert_eq!(repo.current().unwrap(), None);
    repo.save("first prompt").unwrap();
    repo.save("second prompt").unwrap();
    assert_eq!(repo.current().unwrap().as_deref(), Some("second prompt"));
}

#[test]
fn state_get_set_upsert() {
    let (_dir, db) = database();
    let repo = SqliteStateRepository::new(db);
    assert_eq!(repo.get("draft").unwrap(), None);
    repo.set("draft", "hello").unwrap();
    repo.set("draft", "hello world").unwrap();
    assert_eq!(repo.get("draft").unwrap().as_deref(), Some("hello world"));
}

#[test]
fn several_first_launches_at_once_all_succeed() {
    for round in 0..10 {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data/cleanping.db");
        let launches: Vec<_> = (0..8)
            .map(|_| {
                let path = path.clone();
                std::thread::spawn(move || Database::new(path).migrate())
            })
            .collect();
        for launch in launches {
            launch
                .join()
                .unwrap()
                .unwrap_or_else(|e| panic!("round {round}: {e}"));
        }
        let versions: Vec<i64> = Database::new(path)
            .with(|c| {
                c.prepare("SELECT version FROM schema_migrations")?
                    .query_map([], |r| r.get(0))?
                    .collect()
            })
            .unwrap();
        assert_eq!(versions, [1], "round {round}");
    }
}

#[test]
fn an_existing_data_folder_that_others_can_read_is_tightened() {
    let dir = tempfile::tempdir().unwrap();
    let folder = dir.path().join("data");
    std::fs::create_dir(&folder).unwrap();
    std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o755)).unwrap();
    Database::new(folder.join("cleanping.db"))
        .migrate()
        .unwrap();
    let mode = std::fs::metadata(&folder).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o700);
}
