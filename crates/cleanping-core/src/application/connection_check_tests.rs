use super::*;
use crate::application::test_support::{FakeRewriter, FakeSecrets};
use crate::domain::errors::CleanpingError;
use crate::domain::models::Credential;

const REMOTE: &str = "https://api.example.com/v1/chat/completions";
const LOCAL: &str = "http://127.0.0.1:11434/v1/chat/completions";

fn credential(url: &str) -> Credential {
    Credential {
        id: Some(1),
        name: "Work".into(),
        api_url: url.into(),
        model: "test-model".into(),
    }
}

#[test]
fn a_working_key_passes_and_only_the_fixed_test_text_is_sent() {
    let rewriter = FakeRewriter::returning("OK");
    let check = ConnectionCheck::new(rewriter, FakeSecrets::with("Work", "the-key"));
    check.run(&credential(REMOTE)).unwrap();
    let calls = check.rewriter.calls.borrow();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].text, CHECK_TEXT);
    assert_eq!(calls[0].instructions, CHECK_INSTRUCTIONS);
    assert_eq!(calls[0].api_url, REMOTE);
    assert_eq!(calls[0].api_key, "the-key");
    assert_eq!(calls[0].model, "test-model");
}

#[test]
fn a_remote_provider_without_a_key_is_refused_before_any_request() {
    let check = ConnectionCheck::new(FakeRewriter::returning("OK"), FakeSecrets::default());
    let error = check.run(&credential(REMOTE)).unwrap_err();
    assert!(
        matches!(error, CleanpingError::MissingCredential(_)),
        "{error}"
    );
    assert!(error.to_string().contains("Work"));
    assert!(check.rewriter.calls.borrow().is_empty());
}

#[test]
fn a_local_provider_needs_no_key() {
    let check = ConnectionCheck::new(FakeRewriter::returning("OK"), FakeSecrets::default());
    check.run(&credential(LOCAL)).unwrap();
    assert_eq!(check.rewriter.calls.borrow()[0].api_key, "");
}

#[test]
fn a_provider_failure_comes_back_with_its_message() {
    let failing = FakeRewriter::failing(CleanpingError::Rewrite("API returned HTTP 401.".into()));
    let check = ConnectionCheck::new(failing, FakeSecrets::with("Work", "the-key"));
    let error = check.run(&credential(REMOTE)).unwrap_err();
    assert!(matches!(error, CleanpingError::Rewrite(_)));
    assert_eq!(error.to_string(), "API returned HTTP 401.");
}
