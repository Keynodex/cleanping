//! With the shape rule on (the shell key), the text is a command line: a reply that changes the
//! command's quotes, flags or paths is refused like a reply of the wrong shape.

use super::*;
use crate::application::test_support::{FakeRewriter, FakeRuns, FakeSecrets};
use crate::domain::models::RunStatus;

const TYPED: &str = "curl -d '{\"name\": \"lamp\"}";
const CLOSED: &str = "curl -d '{\"name\": \"lamp\"}'";

fn credential() -> Credential {
    Credential {
        id: Some(7),
        name: "OpenAI".into(),
        api_url: "https://api.example.com/v1".into(),
        model: "m".into(),
    }
}

fn polisher(reply: &str) -> PolishText<FakeRewriter, FakeRuns, FakeSecrets> {
    PolishText::new(
        FakeRewriter::returning(reply),
        FakeRuns::default(),
        FakeSecrets::with("OpenAI", "sk-x"),
    )
}

#[test]
fn a_reply_that_closes_the_commands_open_quote_is_refused_and_recorded_as_an_error() {
    let svc = polisher(CLOSED).keeping_shape();
    let result = svc.run(TYPED, "Fix it.", &credential()).unwrap();
    assert_eq!(result.output_text, None);
    assert_eq!(
        result.error_message.as_deref(),
        Some(
            "The reply changed your command (a quote was closed in the command starting \
             `curl -d '{\"name\": \"lamp\"}`); not applied."
        )
    );
    let run = &svc.runs.0.borrow()[0];
    assert_eq!(
        (run.status, run.output_text.clone()),
        (RunStatus::Error, None)
    );
}

#[test]
fn a_dropped_flag_is_refused_too() {
    let svc = polisher("rm -r build/").keeping_shape();
    let result = svc
        .run("rm -r -f build/", "Fix it.", &credential())
        .unwrap();
    let message = result.error_message.unwrap_or_default();
    assert!(message.contains("the flag `-f` was removed"), "{message}");
}

#[test]
fn a_reply_that_keeps_the_command_is_applied() {
    let svc = polisher("curl -d '{\"name\": \"lamp\"}").keeping_shape();
    let result = svc.run("curl -d '{\"nmae\": \"lamp\"}", "Fix it.", &credential());
    assert!(result.unwrap().ok());
}

#[test]
fn without_the_shape_rule_a_changed_command_is_not_refused() {
    let result = polisher(CLOSED)
        .run(TYPED, "Fix it.", &credential())
        .unwrap();
    assert_eq!(result.output_text.as_deref(), Some(CLOSED));
}
