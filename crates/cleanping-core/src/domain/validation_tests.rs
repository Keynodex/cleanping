use super::*;
use crate::domain::errors::CleanpingError;

fn draft(name: &str, url: &str, model: &str) -> CredentialInput {
    CredentialInput {
        name: name.into(),
        api_url: url.into(),
        model: model.into(),
        api_key: "sk-test".into(),
    }
}

fn message(err: CleanpingError) -> String {
    err.to_string()
}

#[test]
fn loopback_forms_are_local() {
    for url in [
        "http://localhost:11434/v1/chat/completions",
        "http://127.0.0.1:11434/v1/chat/completions",
        "http://[::1]:11434/v1/chat/completions",
    ] {
        assert!(is_local_url(url), "{url}");
    }
}

#[test]
fn remote_hosts_and_garbage_are_not_local() {
    assert!(!is_local_url(
        "https://api.deepseek.com/v1/chat/completions"
    ));
    assert!(!is_local_url("not a url"));
}

#[test]
fn accepts_https_and_trims_whitespace() {
    assert_eq!(
        validate_api_url("https://api.openai.com/v1/chat/completions").unwrap(),
        "https://api.openai.com/v1/chat/completions"
    );
    assert_eq!(
        validate_api_url("  https://example.com/v1  ").unwrap(),
        "https://example.com/v1"
    );
}

#[test]
fn accepts_http_only_for_loopback() {
    for host in ["localhost", "127.0.0.1", "[::1]"] {
        let url = format!("http://{host}:11434/v1/chat/completions");
        assert_eq!(validate_api_url(&url).unwrap(), url);
    }
}

#[test]
fn rejects_plain_http_for_remote_host() {
    let err = validate_api_url("http://api.example.com/v1/chat/completions").unwrap_err();
    assert!(message(err).contains("HTTPS"));
}

#[test]
fn rejects_embedded_credentials() {
    let err = validate_api_url("https://user:pass@example.com/v1").unwrap_err();
    assert!(message(err).contains("credentials"));
}

#[test]
fn rejects_missing_host_and_non_http_schemes() {
    assert!(validate_api_url("https:///v1/chat/completions").is_err());
    assert!(validate_api_url("https://").is_err());
    assert!(validate_api_url("ftp://example.com/v1").is_err());
    assert!(validate_api_url("example.com/v1").is_err());
}

#[test]
fn credential_fields_are_normalized() {
    let out = validate_credential_fields(CredentialInput {
        api_key: "  sk-x  ".into(),
        ..draft("  OpenAI  ", "https://api.openai.com/v1", " x ")
    })
    .unwrap();
    assert_eq!(
        (out.name.as_str(), out.model.as_str(), out.api_key.as_str()),
        ("OpenAI", "x", "sk-x")
    );
}

#[test]
fn name_and_model_are_required_and_url_rules_apply() {
    let ok = "https://api.openai.com/v1";
    assert!(
        message(validate_credential_fields(draft("   ", ok, "m")).unwrap_err()).contains("name")
    );
    assert!(message(validate_credential_fields(draft("n", ok, "")).unwrap_err()).contains("model"));
    assert!(validate_credential_fields(draft("n", "http://remote.example.com/v1", "m")).is_err());
}

#[test]
fn names_and_models_must_be_plain_text_on_one_line() {
    for bad in [
        "two\nlines",
        "tab\there",
        "esc\u{1b}[31m",
        "flip\u{202e}ed",
        "zero\u{200b}width",
    ] {
        let as_name = draft(bad, "https://api.example.com/v1", "m");
        let err = validate_credential_fields(as_name).unwrap_err();
        assert!(err.to_string().contains("plain text"), "{bad:?}: {err}");
        let as_model = draft("Ok", "https://api.example.com/v1", bad);
        let err = validate_credential_fields(as_model).unwrap_err();
        assert!(err.to_string().contains("plain text"), "{bad:?}: {err}");
    }
    let fine = draft("caf\u{e9} key", "https://api.example.com/v1", "gpt-4o-mini");
    assert!(validate_credential_fields(fine).is_ok());
}

#[test]
fn debug_output_never_shows_the_api_key() {
    let shown = format!("{:?}", draft("n", "https://a.example/v1", "m"));
    assert!(!shown.contains("sk-test"));
    assert!(shown.contains("<redacted>"));
}

#[test]
fn the_returned_url_is_the_canonical_form_the_http_client_will_use() {
    assert_eq!(
        validate_api_url("HTTPS://Example.COM/v1").unwrap(),
        "https://example.com/v1"
    );
    assert_eq!(
        validate_api_url("http://LOCALHOST:11434/x").unwrap(),
        "http://localhost:11434/x"
    );
    assert_eq!(
        validate_api_url("http://2130706433/x").unwrap(),
        "http://127.0.0.1/x"
    );
}

#[test]
fn same_origin_compares_scheme_host_and_port_but_not_the_path() {
    assert!(same_origin(
        "https://a.example/v1",
        "https://A.example/other"
    ));
    assert!(same_origin(
        "https://a.example:443/v1",
        "https://a.example/v1"
    ));
    assert!(!same_origin("https://a.example/v1", "https://b.example/v1"));
    assert!(!same_origin("https://a.example/v1", "http://a.example/v1"));
    assert!(!same_origin(
        "https://a.example:8443/v1",
        "https://a.example/v1"
    ));
    assert!(!same_origin("not a url", "https://a.example"));
}

#[test]
fn api_keys_must_be_plain_visible_characters() {
    for bad in [
        "sk with space",
        "sk\tx",
        "sk-\u{201c}x\u{201d}",
        "sk-\u{e9}",
        "sk-\u{7}",
    ] {
        let err = validate_credential_fields(CredentialInput {
            api_key: bad.into(),
            ..draft("n", "https://a.example/v1", "m")
        })
        .unwrap_err();
        assert!(message(err).contains("visible"), "{bad:?}");
    }
}

#[test]
fn blank_and_ordinary_keys_pass_and_absurd_ones_do_not() {
    for good in ["", "sk-proj-AbC_123.xyz~+/="] {
        let out = validate_credential_fields(CredentialInput {
            api_key: good.into(),
            ..draft("n", "https://a.example/v1", "m")
        })
        .unwrap();
        assert_eq!(out.api_key, good);
    }
    let err = validate_credential_fields(CredentialInput {
        api_key: "k".repeat(4097),
        ..draft("n", "https://a.example/v1", "m")
    })
    .unwrap_err();
    assert!(message(err).contains("too long"));
}
