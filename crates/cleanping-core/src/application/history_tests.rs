use super::*;
use crate::application::test_support::FakeRuns;
use crate::domain::errors::CleanpingError;
use crate::domain::models::RunStatus;

fn run_at(created_at: &str, input: &str) -> Run {
    Run {
        input_text: input.into(),
        output_text: Some("ok".into()),
        status: RunStatus::Ok,
        error_message: None,
        duration_ms: 1,
        credential_name: None,
        model: "m".into(),
        prompt_text: "p".into(),
        id: None,
        created_at: Some(created_at.into()),
        credential_id: None,
    }
}

fn service(stamps: &[&str]) -> HistoryService<FakeRuns> {
    let runs = FakeRuns::default();
    for (n, stamp) in stamps.iter().enumerate() {
        runs.0.borrow_mut().push(run_at(stamp, &format!("run {n}")));
    }
    HistoryService::new(runs)
}

#[test]
fn recent_is_newest_first_and_limited() {
    let svc = service(&["2026-01-01T00:00:00+00:00", "2026-02-01T00:00:00+00:00"]);
    let inputs: Vec<String> = svc
        .recent(1)
        .unwrap()
        .into_iter()
        .map(|r| r.input_text)
        .collect();
    assert_eq!(inputs, ["run 1"]);
}

#[test]
fn clear_removes_everything_and_reports_the_count() {
    let svc = service(&["2026-01-01T00:00:00+00:00", "2026-02-01T00:00:00+00:00"]);
    assert_eq!(svc.clear().unwrap(), 2);
    assert_eq!(svc.clear().unwrap(), 0);
    assert!(svc.recent(10).unwrap().is_empty());
}

#[test]
fn purge_before_removes_only_older_runs() {
    let svc = service(&[
        "2026-01-01T00:00:00+00:00",
        "2026-03-01T00:00:00+00:00",
        "2026-05-01T00:00:00+00:00",
    ]);
    assert_eq!(svc.purge_before("2026-04-01T00:00:00+00:00").unwrap(), 2);
    let left = svc.recent(10).unwrap();
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].input_text, "run 2");
}

#[test]
fn a_malformed_cutoff_deletes_nothing() {
    let svc = service(&["2026-01-01T00:00:00+00:00"]);
    for bad in ["", "z", "yesterday", "2026-01-01"] {
        let error = svc.purge_before(bad).unwrap_err();
        assert!(matches!(error, CleanpingError::Validation(_)), "{bad}");
    }
    assert_eq!(svc.recent(10).unwrap().len(), 1);
}
