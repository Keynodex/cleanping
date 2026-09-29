//! The two flags the shell key relies on: `--refuse-secrets` (never send a command line that
//! looks like it holds a secret) and `--marks` (which parts of a reply changed).

mod support;

use support::*;

/// Looks like a real API key to the secret check, and is not one. Written in two pieces so no
/// line in this file looks like a real key to a secret scanner.
const FAKE_KEY: &str = concat!("sk-", "Zx81QmVt3LpRw92NcYb47HdKe06Fa5Ug");

fn with_server(reply: &str) -> (Sandbox, FakeServer) {
    let server = serve(vec![ok_reply(reply)]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    (sandbox, server)
}

fn requests(server: &FakeServer) -> usize {
    server.requests.lock().unwrap().len()
}

#[test]
fn refuse_secrets_sends_nothing_for_text_that_looks_like_a_secret() {
    let (sandbox, server) = with_server("Cleaned.");
    let line = format!("export KEY={FAKE_KEY}");
    let out = sandbox.run(&["--refuse-secrets", &line], None);
    assert_eq!(out.code, 2, "must refuse: {}", out.stderr);
    assert_eq!(out.stdout, "");
    assert!(out.stderr.contains("an API key or token"), "{}", out.stderr);
    assert!(out.stderr.contains("not sent"), "{}", out.stderr);
    assert!(
        !out.stderr.contains(FAKE_KEY),
        "the secret must never be echoed"
    );
    assert_eq!(requests(&server), 0);
}

#[test]
fn refuse_secrets_also_covers_text_from_a_pipe() {
    let (sandbox, server) = with_server("Cleaned.");
    let piped = format!("curl -H 'Authorization: Bearer {FAKE_KEY}' https://example.test");
    let out = sandbox.run(&["--refuse-secrets"], Some(&piped));
    assert_eq!(out.code, 2, "must refuse: {}", out.stderr);
    assert_eq!(requests(&server), 0);
}

#[test]
fn refuse_secrets_lets_ordinary_text_through() {
    let (sandbox, server) = with_server("Please fix the login.");
    let out = sandbox.run(&["--refuse-secrets", "pls fix teh login"], None);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert_eq!(out.stdout, "Please fix the login.\n");
    assert_eq!(requests(&server), 1);
}

#[test]
fn without_the_flag_text_is_sent_as_before() {
    let (sandbox, server) = with_server("Cleaned.");
    let line = format!("export KEY={FAKE_KEY}");
    let out = sandbox.run(&[&line], None);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert_eq!(requests(&server), 1);
}

fn marks(input: &str) -> Out {
    Sandbox::new().run(&["--marks"], Some(input))
}

#[test]
fn marks_lists_the_changed_character_ranges_of_the_reply() {
    let out = marks("pleae fix this\0Please fix this.");
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert_eq!(out.stdout, "0 6\n11 16\n");
}

#[test]
fn marks_prints_nothing_when_nothing_changed() {
    let out = marks("fix the login\0fix the login");
    assert_eq!((out.code, out.stdout.as_str()), (0, ""));
}

#[test]
fn marks_counts_characters_not_bytes() {
    let out = marks("cafe ok naive\0café ok naïve");
    assert_eq!(out.stdout, "0 4\n8 13\n");
}

#[test]
fn marks_needs_both_texts() {
    let out = marks("only one text");
    assert_eq!((out.code, out.stdout.as_str()), (2, ""));
    assert!(out.stderr.contains("original"), "{}", out.stderr);
}

#[test]
fn marks_touches_no_database_and_no_files() {
    // The shell runs this on every key press, next to a rewrite that already succeeded.
    let sandbox = Sandbox::new();
    let out = sandbox.run(&["--marks"], Some("a b\0a c"));
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(!sandbox.db_path().exists());
    assert!(!sandbox.dir.path().join("config").exists());
}

#[test]
fn marks_cannot_be_combined_with_a_rewrite() {
    let sandbox = Sandbox::new();
    let out = sandbox.run(&["--marks", "some text"], Some("a\0b"));
    assert_eq!(out.code, 2, "{}", out.stderr);
}
