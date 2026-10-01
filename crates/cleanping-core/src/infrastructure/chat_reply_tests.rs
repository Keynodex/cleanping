use super::*;

fn body(choice: serde_json::Value) -> String {
    serde_json::json!({ "choices": [choice] }).to_string()
}

fn message(error: Result<String>) -> String {
    match error {
        Err(CleanpingError::Rewrite(message)) => message,
        other => panic!("expected a Rewrite error, got {other:?}"),
    }
}

#[test]
fn the_first_choice_is_cleaned_and_returned() {
    let reply = body(serde_json::json!({"message": {"content": "  Fixed\u{1b} text.  "}}));
    assert_eq!(edited_text(&reply).unwrap(), "Fixed text.");
}

#[test]
fn a_body_that_is_not_a_chat_completion_has_no_edited_text() {
    for bad in ["not json", r#"{"unexpected": true}"#, r#"{"choices": []}"#] {
        assert_eq!(
            message(edited_text(bad)),
            "API response did not contain edited text."
        );
    }
}

#[test]
fn an_empty_edit_is_refused() {
    let reply = body(serde_json::json!({"message": {"content": "   "}}));
    assert!(message(edited_text(&reply)).contains("empty edit"));
}
