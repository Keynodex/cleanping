use super::*;
use crate::application::test_support::{FakeRewriter, FakeRuns, FakeSecrets};
use crate::domain::errors::CleanpingError;
use crate::domain::models::RunStatus;

fn credential() -> Credential {
    Credential {
        id: Some(7),
        name: "OpenAI".into(),
        api_url: "https://api.example.com/v1".into(),
        model: "m".into(),
    }
}

fn use_case(
    rewriter: FakeRewriter,
    secrets: FakeSecrets,
) -> PolishText<FakeRewriter, FakeRuns, FakeSecrets> {
    PolishText::new(rewriter, FakeRuns::default(), secrets)
}

#[test]
fn success_records_an_ok_run_and_returns_the_output() {
    let svc = use_case(
        FakeRewriter::returning("Fixed text."),
        FakeSecrets::with("OpenAI", "sk-x"),
    );
    let result = svc.run("pleae fix", "Fix it.", &credential()).unwrap();
    assert!(result.ok());
    assert_eq!(result.output_text.as_deref(), Some("Fixed text."));
    let run = &svc.runs.0.borrow()[0];
    assert_eq!(run.status, RunStatus::Ok);
    assert_eq!(
        (run.credential_name.as_deref(), run.credential_id),
        (Some("OpenAI"), Some(7))
    );
    assert_eq!(run.prompt_text, "Fix it.");
}

#[test]
fn provider_error_records_an_error_run_and_returns_the_message() {
    let err = CleanpingError::Rewrite("API returned HTTP 401.".into());
    let svc = use_case(
        FakeRewriter::failing(err),
        FakeSecrets::with("OpenAI", "sk-x"),
    );
    let result = svc.run("draft", "Fix it.", &credential()).unwrap();
    assert!(!result.ok());
    assert_eq!(
        result.error_message.as_deref(),
        Some("API returned HTTP 401.")
    );
    let run = &svc.runs.0.borrow()[0];
    assert_eq!(
        (run.status, run.output_text.clone()),
        (RunStatus::Error, None)
    );
}

#[test]
fn missing_key_fails_and_records_nothing() {
    let svc = use_case(FakeRewriter::returning("x"), FakeSecrets::default());
    let err = svc.run("draft", "Fix it.", &credential()).unwrap_err();
    assert!(matches!(err, CleanpingError::MissingCredential(_)));
    assert!(svc.runs.0.borrow().is_empty());
}

#[test]
fn local_url_without_a_key_runs_with_an_empty_key() {
    let svc = use_case(FakeRewriter::returning("ok"), FakeSecrets::default());
    let local = Credential {
        id: Some(8),
        name: "Ollama".into(),
        api_url: "http://127.0.0.1:11434/v1/chat/completions".into(),
        model: "qwen2.5:7b".into(),
    };
    assert!(svc.run("draft", "Fix it.", &local).unwrap().ok());
    assert_eq!(svc.rewriter.calls.borrow()[0].api_key, "");
}

#[test]
fn the_key_is_looked_up_by_credential_name() {
    let svc = use_case(
        FakeRewriter::returning("ok"),
        FakeSecrets::with("OpenAI", "sk-secret"),
    );
    svc.run("draft", "Fix it.", &credential()).unwrap();
    assert_eq!(svc.rewriter.calls.borrow()[0].api_key, "sk-secret");
}

#[test]
fn the_rewriter_receives_text_instructions_url_and_model() {
    let svc = use_case(
        FakeRewriter::returning("ok"),
        FakeSecrets::with("OpenAI", "sk-x"),
    );
    svc.run("rough text", "Fix it.", &credential()).unwrap();
    let call = &svc.rewriter.calls.borrow()[0];
    assert_eq!(
        (
            call.text.as_str(),
            call.instructions.as_str(),
            call.api_url.as_str(),
            call.model.as_str()
        ),
        ("rough text", "Fix it.", "https://api.example.com/v1", "m")
    );
}

#[test]
fn a_reply_that_does_not_keep_the_shape_is_refused_and_recorded_as_an_error() {
    let hidden = format!("touch X;{}ls", " ".repeat(3000));
    for reply in ["one\ntwo", hidden.as_str()] {
        let svc = use_case(
            FakeRewriter::returning(reply),
            FakeSecrets::with("OpenAI", "sk-x"),
        )
        .keeping_shape();
        let result = svc.run("ls", "Fix it.", &credential()).unwrap();
        assert_eq!(result.output_text, None);
        assert_eq!(
            result.error_message.as_deref(),
            Some("The reply is longer or has more lines than your text; not applied.")
        );
        let run = &svc.runs.0.borrow()[0];
        assert_eq!(
            (run.status, run.output_text.clone()),
            (RunStatus::Error, None)
        );
    }
}

#[test]
fn a_reply_that_keeps_the_shape_is_kept() {
    let svc = use_case(
        FakeRewriter::returning("Line one.\nLine two."),
        FakeSecrets::with("OpenAI", "sk-x"),
    )
    .keeping_shape();
    let result = svc
        .run("line one\nline two", "Fix it.", &credential())
        .unwrap();
    assert_eq!(result.output_text.as_deref(), Some("Line one.\nLine two."));
}

#[test]
fn without_the_shape_rule_any_reply_is_kept() {
    let svc = use_case(
        FakeRewriter::returning("one\ntwo\nthree"),
        FakeSecrets::with("OpenAI", "sk-x"),
    );
    let result = svc.run("ls", "Fix it.", &credential()).unwrap();
    assert!(result.ok());
}
