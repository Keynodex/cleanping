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

// A provider that stops at its output limit says `finish_reason: "length"`. What it sent is
// part of a rewrite that looks finished, so it must never be used.

const HALF: &str = "The first half of a long rewrite";

fn finished(reason: serde_json::Value, content: serde_json::Value) -> String {
    body(serde_json::json!({"message": {"content": content}, "finish_reason": reason}))
}

fn assert_cut_off(result: Result<String>) {
    match result {
        Err(CleanpingError::Rewrite(message)) => {
            assert!(
                message.starts_with("The reply was cut off"),
                "expected the cut-off refusal, got {message:?}"
            );
            assert!(!message.contains(HALF), "the reply leaked: {message}");
        }
        other => panic!("expected the cut-off refusal, got {other:?}"),
    }
}

#[test]
fn a_reply_cut_off_at_the_length_limit_is_refused() {
    assert_cut_off(edited_text(&finished("length".into(), HALF.into())));
}

#[test]
fn the_length_reason_is_matched_in_any_case() {
    assert_cut_off(edited_text(&finished("LENGTH".into(), HALF.into())));
    assert_cut_off(edited_text(&finished("Length".into(), HALF.into())));
}

#[test]
fn a_cut_off_reply_with_no_text_says_it_was_cut_off_not_empty() {
    // A thinking model can use its whole limit before it writes a word of the answer.
    for content in ["".into(), "   ".into(), serde_json::Value::Null] {
        assert_cut_off(edited_text(&finished("length".into(), content)));
    }
}

#[test]
fn a_finished_reply_is_used_whatever_the_server_calls_the_reason() {
    for reason in ["stop", "end_turn", "STOP", "content_filter", ""] {
        let reply = finished(reason.into(), HALF.into());
        assert_eq!(edited_text(&reply).unwrap(), HALF, "{reason}");
    }
}

#[test]
fn a_reply_with_no_or_a_null_reason_is_used() {
    // Many OpenAI-compatible servers leave the reason out.
    let missing = body(serde_json::json!({"message": {"content": HALF}}));
    assert_eq!(edited_text(&missing).unwrap(), HALF);
    let null = finished(serde_json::Value::Null, HALF.into());
    assert_eq!(edited_text(&null).unwrap(), HALF);
}

#[test]
fn the_cut_off_message_says_what_to_do() {
    let error = edited_text(&finished("length".into(), HALF.into())).unwrap_err();
    let text = error.to_string();
    assert!(text.contains("not used"), "{text}");
    assert!(text.contains("shorter text"), "{text}");
}
