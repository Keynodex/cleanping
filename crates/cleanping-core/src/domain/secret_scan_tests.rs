use super::*;

#[test]
fn private_key_blocks_are_found() {
    for text in [
        "-----BEGIN RSA PRIVATE KEY-----\nMIIE",
        "x -----BEGIN OPENSSH PRIVATE KEY----- y",
        "-----BEGIN PRIVATE KEY-----",
    ] {
        assert_eq!(find_secret(text), Some("a private key"), "{text}");
    }
}

/// Each key shape is written as its well-known prefix plus the rest, joined only when the test
/// runs, so no line in this file looks like a real token to a secret scanner.
#[test]
fn well_known_api_key_shapes_are_found() {
    for (before, prefix, rest) in [
        ("export OPENAI_API_KEY=", "sk-", "abcdefghijklmnopqrstuvwx"),
        ("key is ", "sk-ant-", "api03-abcdefghijklmnopqrstuvwxyz0123"),
        ("", "ghp_", "abcdefghijklmnopqrstuvwxyz0123456789"),
        ("token ", "github_pat_", "11ABCDEFG0123456789_abcdefghij"),
        ("aws ", "AKIA", "IOSFODNN7EXAMPLE here"),
        ("slack ", "xox", "b-1234567890-abcdefghijklmnop"),
        ("google ", "AIza", "SyA-abcdefghijklmnopqrstuvwxyz012345"),
        ("stripe ", "sk_live_", "abcdefghijklmnop"),
        ("", "glpat-", "abcdefghijklmnopqrstuvwx"),
    ] {
        let text = format!("{before}{prefix}{rest}");
        assert_eq!(find_secret(&text), Some("an API key or token"), "{text}");
    }
}

#[test]
fn a_bearer_token_is_found() {
    let text = "curl -H 'Authorization: Bearer abcdefghijklmnopqrstuvwxyz012345'";
    assert_eq!(find_secret(text), Some("an API key or token"));
}

#[test]
fn passwords_and_secrets_assigned_a_value_are_found() {
    for text in [
        "password=hunter2secret",
        "PASSWORD: 'Hunter2Secret'",
        "DB_PASSWORD=abc12345",
        "{\"client_secret\": \"Zx9-abcdef\"}",
        "api_key=abcdef123456",
        "GITHUB_TOKEN=abc123def456",
        "passwd = p@ssw0rd!",
    ] {
        let found = find_secret(text);
        assert!(found.is_some(), "{text}");
    }
    assert_eq!(
        find_secret("password=hunter2secret"),
        Some("a password or secret")
    );
}

#[test]
fn ordinary_writing_and_code_are_not_flagged() {
    for text in [
        "Please fix the login page; the password field is too small",
        "passwords must be at least 8 characters",
        "the risk-averse mask-that-is-long-enough plan",
        "task-force sk-8 is short",
        "let key = compute(token_count);",
        "commit b783f45a43aa8b1f5d4ee35d7b296a6c71438949 fixes it",
        "reset link https://example.com/reset?token=abc",
        "New password: please choose one",
        "enter your api key below",
        "",
    ] {
        assert_eq!(find_secret(text), None, "{text}");
    }
}

#[test]
fn the_label_never_contains_the_secret() {
    let label = find_secret("password=hunter2secret").unwrap();
    assert!(!label.contains("hunter"));
}
