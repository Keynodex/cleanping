use super::*;
use crate::domain::errors::CleanpingError;
use crate::infrastructure::fake_api::{ok_json, serve, serve_raw, status, FakeServer};
use std::net::TcpListener;

const OK_BODY: &str =
    r#"{"choices":[{"message":{"role":"assistant","content":"  Fixed text.  "}}]}"#;

fn request<'a>(api_url: &'a str, api_key: &'a str) -> RewriteRequest<'a> {
    RewriteRequest {
        text: "  pleae fix  ",
        instructions: "Fix it.",
        api_url,
        api_key,
        model: "m",
    }
}

fn rewrite(server: &FakeServer) -> Result<String> {
    OpenAiRewriter::new(Duration::from_secs(5)).rewrite(&request(&server.url, "sk-x"))
}

#[test]
fn rejects_an_unsafe_url_before_any_request() {
    let err = OpenAiRewriter::default()
        .rewrite(&request(
            "http://remote.example.com/v1/chat/completions",
            "sk-x",
        ))
        .unwrap_err();
    assert!(matches!(err, CleanpingError::Validation(_)));
}

#[test]
fn returns_trimmed_content() {
    assert_eq!(
        rewrite(&serve(vec![ok_json(OK_BODY)])).unwrap(),
        "Fixed text."
    );
}

#[test]
fn sends_the_system_prompt_user_text_and_bearer_key() {
    let server = serve(vec![ok_json(OK_BODY)]);
    rewrite(&server).unwrap();
    let request = server.requests.lock().unwrap()[0].clone();
    assert!(request
        .to_ascii_lowercase()
        .contains("authorization: bearer sk-x"));
    let body: serde_json::Value =
        serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
    assert_eq!(body["model"], "m");
    assert_eq!(
        body["messages"][0],
        serde_json::json!({"role": "system", "content": "Fix it."})
    );
    assert_eq!(
        body["messages"][1],
        serde_json::json!({"role": "user", "content": "  pleae fix  "})
    );
}

#[test]
fn http_errors_map_to_a_safe_message_without_the_body() {
    let err = rewrite(&serve(vec![status(
        401,
        "Unauthorized",
        "secret server detail",
    )]))
    .unwrap_err();
    assert_eq!(
        err,
        CleanpingError::Rewrite("API returned HTTP 401.".into())
    );
}

#[test]
fn malformed_payload_and_empty_edit_map_to_rewrite_errors() {
    let malformed = rewrite(&serve(vec![ok_json(r#"{"unexpected": true}"#)])).unwrap_err();
    assert!(malformed.to_string().contains("edited text"));
    let empty = rewrite(&serve(vec![ok_json(
        r#"{"choices":[{"message":{"content":"   "}}]}"#,
    )]))
    .unwrap_err();
    assert!(empty.to_string().contains("empty"));
}

#[test]
fn a_redirect_is_never_followed_so_the_key_and_text_stay_put() {
    let elsewhere = serve(vec![ok_json(OK_BODY)]);
    let redirect = format!(
        "HTTP/1.1 302 Found\r\nLocation: {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        elsewhere.url
    );
    let err = rewrite(&serve(vec![redirect])).unwrap_err();
    assert!(matches!(err, CleanpingError::Rewrite(_)));
    std::thread::sleep(Duration::from_millis(200));
    assert!(
        elsewhere.requests.lock().unwrap().is_empty(),
        "the redirect target was contacted"
    );
}

#[test]
fn a_connection_that_is_dropped_maps_to_a_could_not_reach_message() {
    // A server that hangs up on everyone: a failure that cannot depend on which ports are
    // free (a port that was merely closed can be handed to another test a moment later).
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!(
        "http://127.0.0.1:{}/v1/chat/completions",
        listener.local_addr().unwrap().port()
    );
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            drop(stream);
        }
    });
    let err = OpenAiRewriter::new(Duration::from_secs(2))
        .rewrite(&request(&url, ""))
        .unwrap_err();
    assert!(
        err.to_string().starts_with("Could not reach the API"),
        "{err}"
    );
}

#[test]
fn an_oversized_response_body_is_refused_not_read_into_memory() {
    let huge = "x".repeat(1_200_000);
    let body = serde_json::json!({"choices": [{"message": {"content": huge}}]}).to_string();
    let err = rewrite(&serve(vec![ok_json(&body)])).unwrap_err();
    assert!(err.to_string().contains("too large"), "{err}");
}

fn reply_with(content: &str) -> String {
    let body = serde_json::json!({"choices": [{"message": {"content": content}}]}).to_string();
    ok_json(&body)
}

#[test]
fn terminal_control_characters_in_the_reply_are_removed() {
    let hostile = "\u{1b}]52;c;ZWNobw==\u{7}Fixed\u{202e} text.\r\n";
    let out = rewrite(&serve(vec![reply_with(hostile)])).unwrap();
    assert_eq!(out, "]52;c;ZWNobw==Fixed text.");
    assert!(!out.chars().any(|c| c.is_control()));
}

#[test]
fn a_reply_that_is_only_control_characters_counts_as_empty() {
    let err = rewrite(&serve(vec![reply_with("\u{1b}\u{7}  ")])).unwrap_err();
    assert!(err.to_string().contains("empty"), "{err}");
}

#[test]
fn the_client_does_not_ask_for_compressed_responses() {
    let server = serve(vec![ok_json(OK_BODY)]);
    rewrite(&server).unwrap();
    let request = server.requests.lock().unwrap()[0].to_ascii_lowercase();
    assert!(!request.contains("accept-encoding"), "{request}");
}

#[test]
fn a_gzip_bomb_is_not_expanded_in_memory() {
    use std::io::Write as _;
    let payload = format!(
        r#"{{"choices":[{{"message":{{"content":"{}"}}}}]}}"#,
        "x".repeat(60_000_000)
    );
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
    encoder.write_all(payload.as_bytes()).unwrap();
    let compressed = encoder.finish().unwrap();
    assert!(
        compressed.len() < 200_000,
        "the bomb must be small on the wire"
    );
    let mut response = format!(
        "HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        compressed.len()
    )
    .into_bytes();
    response.extend_from_slice(&compressed);
    let result = rewrite(&serve_raw(vec![response]));
    assert!(
        result.is_err(),
        "a compressed reply must not be inflated and accepted"
    );
}
