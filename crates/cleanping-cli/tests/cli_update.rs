//! `cleanping update`: the outcomes of a check, against a fake release server on this machine.
//! Every run sets `CLEANPING_UPDATE_URL` to a loopback address, so no test can reach GitHub.

mod support;

use support::{ok_reply, serve, Out, Sandbox};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NOT_UNDERSTOOD: &str =
    "cleanping: Could not check for updates: the reply from GitHub was not understood.\n";

fn release_reply(body: &serde_json::Value) -> String {
    let body = body.to_string();
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn release(tag: &str) -> String {
    release_reply(&serde_json::json!({
        "tag_name": tag,
        "html_url": format!("https://github.com/Keynodex/cleanping/releases/tag/{tag}"),
    }))
}

fn check(args: &[&str], reply: String) -> Out {
    let server = serve(vec![reply]);
    Sandbox::new().run_with_env(args, None, &[("CLEANPING_UPDATE_URL", &server.url)])
}

#[test]
fn the_same_version_is_the_latest_and_exits_0() {
    for args in [&["update"][..], &["update", "--check"][..]] {
        let out = check(args, release(&format!("v{VERSION}")));
        assert_eq!(out.code, 0, "{args:?}: {}", out.stderr);
        assert_eq!(
            out.stdout,
            format!("CleanPing {VERSION} is the latest version.\n")
        );
        assert_eq!(out.stderr, "");
    }
}

#[test]
fn a_newer_release_shows_its_page_and_how_to_install_it() {
    let out = check(&["update"], release("v999.0.0"));
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(
        out.stdout.starts_with(&format!(
            "CleanPing 999.0.0 is available (you have {VERSION}).\n"
        )),
        "{}",
        out.stdout
    );
    assert!(out
        .stdout
        .contains("Release page: https://github.com/Keynodex/cleanping/releases/tag/v999.0.0\n"));
    if cfg!(target_os = "macos") {
        assert!(
            out.stdout
                .contains("  shasum -a 256 -c cleanping-v999.0.0-"),
            "{}",
            out.stdout
        );
    } else {
        assert!(
            out.stdout
                .contains("https://github.com/Keynodex/cleanping#install"),
            "{}",
            out.stdout
        );
    }
    assert!(out
        .stdout
        .contains("Nothing was changed: this command only checks for a newer version."));
}

#[test]
fn a_build_newer_than_the_latest_release_is_called_a_development_build() {
    let out = check(&["update"], release("v0.0.1"));
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert_eq!(
        out.stdout,
        format!(
            "CleanPing {VERSION} is newer than the latest release (0.0.1), so this is a \
             development build. Nothing was changed.\n"
        )
    );
}

#[test]
fn nothing_listening_is_a_plain_message_and_exit_1() {
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port(); // the listener is dropped here, so nothing answers on this port
    let url = format!("http://127.0.0.1:{port}/releases/latest");
    let out = Sandbox::new().run_with_env(&["update"], None, &[("CLEANPING_UPDATE_URL", &url)]);
    assert_eq!(out.code, 1);
    assert_eq!(out.stdout, "");
    assert_eq!(
        out.stderr,
        "cleanping: Could not check for updates: GitHub could not be reached. Check your \
         connection and try again.\n"
    );
}

#[test]
fn a_reply_that_is_not_release_json_is_not_understood_and_not_echoed() {
    for body in [
        "<html>private-body-marker</html>".to_string(),
        serde_json::json!({"message": "private-body-marker"}).to_string(),
        serde_json::json!({"tag_name": 5}).to_string(),
    ] {
        let reply = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let out = check(&["update"], reply);
        assert_eq!(out.code, 1, "{body}");
        assert_eq!(out.stdout, "");
        assert_eq!(out.stderr, NOT_UNDERSTOOD, "{body}");
    }
}

#[test]
fn a_pre_release_tag_is_not_understood() {
    let out = check(&["update"], release("v999.0.0-rc.1"));
    assert_eq!(out.code, 1);
    assert_eq!(out.stderr, NOT_UNDERSTOOD);
}

#[test]
fn an_error_status_names_the_code_but_never_the_body() {
    let body = "private-body-marker";
    let reply = format!(
        "HTTP/1.1 403 Forbidden\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let out = check(&["update"], reply);
    assert_eq!(out.code, 1);
    assert_eq!(
        out.stderr,
        "cleanping: Could not check for updates: GitHub answered with HTTP 403.\n"
    );
}

#[test]
fn text_that_starts_with_update_needs_a_double_dash() {
    let server = serve(vec![ok_reply("Update the docs.")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let out = sandbox.run(&["--", "update", "the", "docs"], None);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert_eq!(out.stdout, "Update the docs.\n");
    assert_eq!(server.user_text(0), "update the docs");
    let dead = "http://127.0.0.1:9/never"; // in case `update` ever ran, it must not leave here
    let bare = sandbox.run_with_env(
        &["update", "the", "docs"],
        None,
        &[("CLEANPING_UPDATE_URL", dead)],
    );
    assert_eq!(
        bare.code, 2,
        "update is a command word, so extra words are refused"
    );
    assert_eq!(
        server.requests.lock().unwrap().len(),
        1,
        "nothing more was sent"
    );
}
