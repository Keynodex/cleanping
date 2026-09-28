//! What the CLI does with a hostile or odd reply, proxies, and words that look like commands.

mod support;

use support::*;

#[test]
fn control_characters_in_a_reply_never_reach_the_terminal() {
    let server = serve(vec![ok_reply("Hello\u{1b}[31m red\u{7}\u{8}!\r\nnext")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let out = sandbox.run(&["hi"], None);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert_eq!(out.stdout, "Hello[31m red!\nnext\n");
}

#[test]
fn keep_shape_refuses_extra_lines_and_padding_and_prints_nothing() {
    let padded = format!("touch X;{}ls", " ".repeat(3000));
    let server = serve(vec![
        ok_reply("one\ntwo\nthree"),
        ok_reply(&padded),
        ok_reply("Hello there."),
    ]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    for _ in 0..2 {
        let refused = sandbox.run(&["--keep-shape", "hi"], None);
        assert_eq!((refused.code, refused.stdout.as_str()), (1, ""));
        assert!(
            refused
                .stderr
                .contains("longer or has more lines than your text"),
            "{}",
            refused.stderr
        );
    }
    let allowed = sandbox.run(&["--keep-shape", "hi"], None);
    assert_eq!(
        (allowed.code, allowed.stdout.as_str()),
        (0, "Hello there.\n")
    );
}

#[test]
fn without_keep_shape_a_multi_line_reply_is_fine() {
    let server = serve(vec![ok_reply("one\ntwo\nthree")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let out = sandbox.run(&["hi"], None);
    assert_eq!((out.code, out.stdout.as_str()), (0, "one\ntwo\nthree\n"));
}

#[test]
fn a_refused_reply_is_kept_in_the_history_as_an_error_not_a_success() {
    let server = serve(vec![ok_reply("one\ntwo")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    sandbox.run(&["--keep-shape", "hi"], None);
    let listed = sandbox.run(&["history", "list"], None);
    assert!(listed.stdout.contains(" error "), "{}", listed.stdout);
    assert!(!listed.stdout.contains(" ok "), "{}", listed.stdout);
}

fn add_remote_key(sandbox: &Sandbox) {
    let added = sandbox.run(
        &[
            "keys",
            "add",
            "--name",
            "Remote",
            "--url",
            "https://remote.example.com/v1/chat/completions",
            "--model",
            "m",
            "--key-stdin",
        ],
        Some("sk-remote\n"),
    );
    assert_eq!(added.code, 0, "{}", added.stderr);
}

#[test]
fn an_unusable_proxy_setting_stops_a_remote_request_before_anything_is_sent() {
    let sandbox = Sandbox::new();
    add_remote_key(&sandbox);
    let bad = "socks5://user:secret-pw@127.0.0.1:1";
    let out = sandbox.run_with_env(&["hi"], None, &[("HTTPS_PROXY", bad)]);
    assert_eq!((out.code, out.stdout.as_str()), (1, ""));
    assert!(out.stderr.contains("HTTPS_PROXY"), "{}", out.stderr);
    assert!(out.stderr.contains("nothing was sent"), "{}", out.stderr);
    assert!(!out.stderr.contains("secret-pw"), "{}", out.stderr);
}

#[test]
fn a_local_api_ignores_an_unusable_proxy_setting() {
    let server = serve(vec![ok_reply("direct")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let out = sandbox.run_with_env(&["hi"], None, &[("ALL_PROXY", "socks5://127.0.0.1:1")]);
    assert_eq!(
        (out.code, out.stdout.as_str()),
        (0, "direct\n"),
        "{}",
        out.stderr
    );
}

#[test]
fn an_error_keeps_its_own_exit_code_when_stderr_is_closed() {
    let out = Sandbox::new().run_with_closed_stderr(&["hello"]);
    assert_eq!(out.code, 3, "no saved key must still exit 3, not panic");
}

#[test]
fn the_first_word_help_is_text_to_rewrite_not_a_subcommand() {
    let server = serve(vec![ok_reply("Help me fix this.")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let out = sandbox.run(&["help", "me", "fix", "this"], None);
    assert_eq!((out.code, out.stdout.as_str()), (0, "Help me fix this.\n"));
    assert_eq!(server.user_text(0), "help me fix this");
    assert!(sandbox.run(&["--help"], None).stdout.contains("Usage"));
}

#[test]
fn a_local_api_is_reached_directly_even_when_a_proxy_is_configured() {
    let api = serve(vec![ok_reply("direct")]);
    let proxy = serve(vec![ok_reply("via proxy")]);
    let proxy_url = proxy
        .url
        .trim_end_matches("/v1/chat/completions")
        .to_string();
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &api);
    let out = sandbox.run_with_env(
        &["hi"],
        None,
        &[
            ("HTTP_PROXY", &proxy_url),
            ("http_proxy", &proxy_url),
            ("ALL_PROXY", &proxy_url),
        ],
    );
    assert_eq!(
        (out.code, out.stdout.as_str()),
        (0, "direct\n"),
        "{}",
        out.stderr
    );
    assert!(proxy.requests.lock().unwrap().is_empty());
}

#[test]
fn a_remote_api_does_go_through_the_configured_proxy() {
    let proxy = serve(vec![status_reply(502, "")]);
    let proxy_url = proxy
        .url
        .trim_end_matches("/v1/chat/completions")
        .to_string();
    let sandbox = Sandbox::new();
    let added = sandbox.run(
        &[
            "keys",
            "add",
            "--name",
            "Remote",
            "--url",
            "https://remote.example.com/v1/chat/completions",
            "--model",
            "m",
            "--key-stdin",
        ],
        Some("sk-remote\n"),
    );
    assert_eq!(added.code, 0, "{}", added.stderr);
    let out = sandbox.run_with_env(
        &["hi"],
        None,
        &[("HTTPS_PROXY", &proxy_url), ("https_proxy", &proxy_url)],
    );
    assert_eq!(out.code, 1, "{}", out.stderr);
    let seen = proxy.requests.lock().unwrap().clone();
    assert!(
        seen.first()
            .is_some_and(|r| r.starts_with("CONNECT remote.example.com:443")),
        "{seen:?}"
    );
    assert!(!out.stderr.contains("sk-remote"));
}
