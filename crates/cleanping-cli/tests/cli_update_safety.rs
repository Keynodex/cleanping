//! `cleanping update` treats the release server as untrusted and tells it nothing about you.
//! Every run sets `CLEANPING_UPDATE_URL` to a loopback address, so no test can reach GitHub.

mod support;

use support::{serve, FakeServer, Out, Sandbox};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn reply_with(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn check(reply: String) -> (FakeServer, Sandbox, Out) {
    let server = serve(vec![reply]);
    let sandbox = Sandbox::new();
    let out = sandbox.run_with_env(&["update"], None, &[("CLEANPING_UPDATE_URL", &server.url)]);
    (server, sandbox, out)
}

#[test]
fn the_request_carries_only_host_and_user_agent() {
    let body = serde_json::json!({"tag_name": format!("v{VERSION}")}).to_string();
    let (server, _sandbox, out) = check(reply_with(&body));
    assert_eq!(out.code, 0, "{}", out.stderr);
    let requests = server.requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    let (head, rest) = requests[0].split_once("\r\n\r\n").unwrap();
    assert_eq!(rest, "", "no body is sent");
    let mut lines = head.lines();
    assert_eq!(lines.next().unwrap(), "GET /v1/chat/completions HTTP/1.1");
    let mut headers: Vec<(String, String)> = lines
        .map(|line| {
            let (name, value) = line.split_once(':').unwrap();
            (name.to_ascii_lowercase(), value.trim().to_string())
        })
        .collect();
    headers.sort();
    assert_eq!(headers.len(), 2, "{headers:?}");
    assert_eq!(headers[0].0, "host");
    assert_eq!(
        headers[1],
        ("user-agent".into(), format!("cleanping/{VERSION}"))
    );
}

#[test]
fn an_oversized_reply_is_refused() {
    let padding = "x".repeat(300 * 1024);
    let body = serde_json::json!({"tag_name": "v999.0.0", "body": padding}).to_string();
    let (_server, _sandbox, out) = check(reply_with(&body));
    assert_eq!(out.code, 1);
    assert_eq!(
        out.stderr,
        "cleanping: Could not check for updates: the reply from GitHub was too large.\n"
    );
}

#[test]
fn a_release_page_off_the_cleanping_repository_is_replaced_by_the_standard_one() {
    let body = serde_json::json!({
        "tag_name": "v999.0.0",
        "html_url": "https://evil.example/Keynodex/cleanping/releases/tag/v999.0.0",
    })
    .to_string();
    let (_server, _sandbox, out) = check(reply_with(&body));
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(out
        .stdout
        .contains("Release page: https://github.com/Keynodex/cleanping/releases/latest\n"));
    assert!(!out.stdout.contains("evil"), "{}", out.stdout);
}

#[test]
fn control_characters_in_the_reply_never_reach_the_terminal() {
    let page = "https://github.com/Keynodex/cleanping/\u{1b}]8;;https://evil.example\u{7}x";
    let body = serde_json::json!({"tag_name": "v999.0.0", "html_url": page}).to_string();
    let (_server, _sandbox, out) = check(reply_with(&body));
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(out.stdout.contains("releases/latest"), "{}", out.stdout);
    let tag = serde_json::json!({"tag_name": "v999.0.0\u{1b}[2J"}).to_string();
    let (_server, _sandbox, refused) = check(reply_with(&tag));
    assert_eq!(refused.code, 1);
    for text in [&out.stdout, &out.stderr, &refused.stdout, &refused.stderr] {
        assert!(
            !text.chars().any(|c| c.is_control() && c != '\n'),
            "{text:?}"
        );
    }
}

#[test]
fn a_redirect_is_refused_and_not_followed() {
    let second = serve(vec![reply_with(r#"{"tag_name":"v999.0.0"}"#)]);
    let redirect = format!(
        "HTTP/1.1 302 Found\r\nLocation: {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        second.url
    );
    let (_server, _sandbox, out) = check(redirect);
    assert_eq!(out.code, 1);
    assert_eq!(
        out.stderr,
        "cleanping: Could not check for updates: GitHub redirected the request; refusing to \
         follow it.\n"
    );
    assert!(
        second.requests.lock().unwrap().is_empty(),
        "the redirect was followed"
    );
}

#[test]
fn a_loopback_check_ignores_proxy_settings() {
    let body = serde_json::json!({"tag_name": format!("v{VERSION}")}).to_string();
    let server = serve(vec![reply_with(&body)]);
    let out = Sandbox::new().run_with_env(
        &["update"],
        None,
        &[
            ("CLEANPING_UPDATE_URL", &server.url),
            ("HTTP_PROXY", "http://127.0.0.1:9"),
            ("ALL_PROXY", "http://127.0.0.1:9"),
        ],
    );
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert_eq!(server.requests.lock().unwrap().len(), 1);
}

#[test]
fn a_check_writes_no_files() {
    let body = serde_json::json!({"tag_name": format!("v{VERSION}")}).to_string();
    let (_server, sandbox, out) = check(reply_with(&body));
    assert_eq!(out.code, 0, "{}", out.stderr);
    let entries: Vec<_> = std::fs::read_dir(sandbox.dir.path()).unwrap().collect();
    assert!(entries.is_empty(), "update wrote {entries:?}");
    // Positive control: a command that does use storage leaves files in the same place.
    sandbox.run(&["keys", "list"], None);
    assert!(
        sandbox.db_path().exists(),
        "keys list should create the database"
    );
}
