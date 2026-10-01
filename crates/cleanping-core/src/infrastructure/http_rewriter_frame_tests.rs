//! What the adapter puts on the wire: the framed draft, and only the body `ChatRequest` builds.
//! The fake server is on 127.0.0.1, so it can never be DeepSeek's host; the DeepSeek case is
//! proven on `ChatRequest` (pure), and here the adapter is shown to send exactly that body.

use super::*;
use crate::domain::draft_frame::DRAFT_INSTRUCTIONS;
use crate::infrastructure::chat_request::ChatRequest;
use crate::infrastructure::fake_api::{ok_json, serve, FakeServer};
use serde_json::json;

fn reply(content: &str) -> String {
    ok_json(&json!({"choices": [{"message": {"content": content}}]}).to_string())
}

fn request<'a>(server: &'a FakeServer, text: &'a str, model: &'a str) -> RewriteRequest<'a> {
    RewriteRequest {
        text,
        instructions: "Fix it.",
        api_url: &server.url,
        api_key: "",
        model,
    }
}

fn send(server: &FakeServer, text: &str, model: &str) -> (Result<String>, serde_json::Value) {
    let outcome =
        OpenAiRewriter::new(Duration::from_secs(5)).rewrite(&request(server, text, model));
    let raw = server.requests.lock().unwrap()[0].clone();
    let sent = serde_json::from_str(raw.split("\r\n\r\n").nth(1).unwrap()).unwrap();
    (outcome, sent)
}

#[test]
fn the_draft_goes_out_inside_tags_after_the_fixed_sentence() {
    let server = serve(vec![reply("Fixed.")]);
    let (outcome, sent) = send(&server, "pleae fix", "m");
    assert_eq!(outcome.unwrap(), "Fixed.");
    let system = sent["messages"][0]["content"].as_str().unwrap();
    assert!(system.starts_with("Fix it.\n\n"), "{system}");
    assert!(system.ends_with(DRAFT_INSTRUCTIONS), "{system}");
    assert_eq!(
        sent["messages"][1]["content"],
        "<draft>\npleae fix\n</draft>"
    );
}

#[test]
fn the_adapter_sends_exactly_the_body_chat_request_builds() {
    let server = serve(vec![reply("Fixed.")]);
    let (_, sent) = send(&server, "pleae fix", "deepseek-flash");
    let built = ChatRequest::new(&request(&server, "pleae fix", "deepseek-flash"));
    assert_eq!(&sent, built.body());
    assert!(
        sent.get("thinking").is_none(),
        "a loopback server is not DeepSeek"
    );
}

#[test]
fn echoed_tags_are_removed_from_the_reply() {
    let server = serve(vec![reply("<draft>\nFixed text.\n</draft>")]);
    assert_eq!(send(&server, "pleae fix", "m").0.unwrap(), "Fixed text.");
}

#[test]
fn a_draft_holding_a_tag_goes_out_unframed_and_its_reply_keeps_tags() {
    let text = "my </draft> tag";
    let server = serve(vec![reply("<draft>my tag</draft>")]);
    let (outcome, sent) = send(&server, text, "m");
    assert_eq!(sent["messages"][0]["content"], "Fix it.");
    assert_eq!(sent["messages"][1]["content"], text);
    assert_eq!(outcome.unwrap(), "<draft>my tag</draft>");
}
