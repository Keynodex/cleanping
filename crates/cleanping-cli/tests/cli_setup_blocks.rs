//! `keys test` (does a saved provider answer?) and the ready-made system prompts.

mod support;

use std::net::TcpListener;

use support::*;

fn one_key(reply: &str) -> (Sandbox, FakeServer) {
    let server = serve(vec![ok_reply(reply)]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    (sandbox, server)
}

fn requests(server: &FakeServer) -> usize {
    server.requests.lock().unwrap().len()
}

#[test]
fn keys_test_reports_a_working_provider_and_sends_only_fixed_words() {
    let (sandbox, server) = one_key("OK");
    let out = sandbox.run(&["keys", "test"], None);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(
        out.stdout
            .starts_with("OK: \u{201c}Local\u{201d} answered in "),
        "{}",
        out.stdout
    );
    assert!(out.stdout.contains("model test-model"), "{}", out.stdout);
    assert_eq!(server.user_text(0), "ping");
    assert!(server.system_text(0).contains("connection test"));
}

#[test]
fn keys_test_saves_nothing_to_the_history() {
    let server = serve(vec![ok_reply("OK"), ok_reply("Cleaned.")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    assert_eq!(sandbox.run(&["keys", "test"], None).code, 0);
    let history = sandbox.run(&["history", "list"], None);
    assert!(!history.stdout.contains("ping"), "{}", history.stdout);
    // The same list does show an ordinary rewrite, so the empty result above means something.
    assert_eq!(sandbox.run(&["ping pong"], None).code, 0);
    let history = sandbox.run(&["history", "list"], None);
    assert!(history.stdout.contains("ping pong"), "{}", history.stdout);
}

#[test]
fn keys_test_names_the_key_and_the_problem_without_the_provider_body() {
    let server = serve(vec![status_reply(401, "provider-said-secret-detail")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let out = sandbox.run(&["keys", "test"], None);
    assert_eq!(out.code, 1);
    assert_eq!(out.stdout, "");
    assert!(
        out.stderr.contains("\u{201c}Local\u{201d}") && out.stderr.contains("HTTP 401"),
        "{}",
        out.stderr
    );
    assert!(!out.stderr.contains("provider-said-secret-detail"));
}

#[test]
fn keys_test_without_a_saved_key_exits_3_and_says_how_to_add_one() {
    let sandbox = Sandbox::new();
    sandbox.insert_credential("Remote", "https://api.example.com/v1/chat/completions");
    let out = sandbox.run(&["keys", "test"], None);
    assert_eq!(out.code, 3, "{}", out.stderr);
    assert!(
        out.stderr.contains("No API key saved") && out.stderr.contains("keys add"),
        "{}",
        out.stderr
    );
}

#[test]
fn keys_test_can_name_a_key_and_never_guesses() {
    let (sandbox, server) = one_key("OK");
    sandbox.insert_credential("Other", "https://api.example.com/v1/chat/completions");
    let named = sandbox.run(&["keys", "test", "Local"], None);
    assert_eq!(named.code, 0, "{}", named.stderr);
    let unknown = sandbox.run(&["keys", "test", "Nope"], None);
    assert_eq!(unknown.code, 3, "{}", unknown.stderr);
    assert!(unknown.stderr.contains("keys list"), "{}", unknown.stderr);
    assert_eq!(requests(&server), 1);
}

#[test]
fn keys_test_tells_you_when_ollama_lacks_the_model() {
    let tags = r#"{"models":[{"name":"llama3:8b"}]}"#;
    let server = serve(vec![
        status_reply(404, "model not found"),
        status_reply(200, tags),
    ]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let out = sandbox.run(&["keys", "test"], None);
    assert_eq!(out.code, 1);
    assert!(
        out.stderr.contains("does not have the model"),
        "{}",
        out.stderr
    );
    assert!(
        out.stderr.contains("ollama pull test-model"),
        "{}",
        out.stderr
    );
}

#[test]
fn keys_test_gives_no_ollama_advice_for_a_server_that_is_not_ollama() {
    let server = serve(vec![status_reply(500, "boom"), status_reply(404, "no")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let out = sandbox.run(&["keys", "test"], None);
    assert_eq!(out.code, 1);
    assert!(out.stderr.contains("HTTP 500"), "{}", out.stderr);
    assert!(
        !out.stderr.to_lowercase().contains("ollama"),
        "{}",
        out.stderr
    );
}

#[test]
fn keys_test_gives_no_install_advice_on_an_address_that_is_not_the_default() {
    let port = {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };
    let sandbox = Sandbox::new();
    let url = format!("http://127.0.0.1:{port}/v1/chat/completions");
    sandbox.insert_credential("Local", &url);
    let out = sandbox.run(&["keys", "test"], None);
    assert_eq!(out.code, 1);
    assert!(
        out.stderr.contains("Could not reach the API"),
        "{}",
        out.stderr
    );
    assert!(
        !out.stderr.to_lowercase().contains("ollama"),
        "{}",
        out.stderr
    );
}

#[test]
fn prompt_presets_lists_every_preset_and_marks_the_one_in_use() {
    let out = Sandbox::new().run(&["prompt", "presets"], None);
    assert_eq!(out.code, 0, "{}", out.stderr);
    for name in ["default", "typos", "concise", "friendly", "structure"] {
        assert!(out.stdout.contains(name), "{}", out.stdout);
    }
    let marked: Vec<&str> = out.stdout.lines().filter(|l| l.starts_with('*')).collect();
    assert_eq!(marked.len(), 1, "{}", out.stdout);
    assert!(marked[0].contains("default"));
}

#[test]
fn prompt_use_switches_the_prompt_and_the_marker_follows() {
    let sandbox = Sandbox::new();
    let used = sandbox.run(&["prompt", "use", "concise"], None);
    assert_eq!(used.code, 0, "{}", used.stderr);
    assert!(sandbox
        .run(&["prompt", "show"], None)
        .stdout
        .contains("concise copy editor"));
    let listed = sandbox.run(&["prompt", "presets"], None).stdout;
    let marked: Vec<&str> = listed.lines().filter(|l| l.starts_with('*')).collect();
    assert_eq!(marked.len(), 1, "{listed}");
    assert!(marked[0].contains("concise"), "{listed}");
    // Another ready-made prompt is in use, so switching again needs no confirmation.
    assert_eq!(sandbox.run(&["prompt", "use", "typos"], None).code, 0);
}

#[test]
fn prompt_use_never_replaces_your_own_prompt_without_yes() {
    let sandbox = Sandbox::new();
    assert_eq!(
        sandbox
            .run(&["prompt", "set", "Answer like a pirate."], None)
            .code,
        0
    );
    let refused = sandbox.run(&["prompt", "use", "typos"], None);
    assert_eq!(refused.code, 2, "{}", refused.stderr);
    assert!(refused.stderr.contains("--yes"), "{}", refused.stderr);
    assert_eq!(
        sandbox.run(&["prompt", "show"], None).stdout.trim(),
        "Answer like a pirate."
    );
    let replaced = sandbox.run(&["prompt", "use", "typos", "--yes"], None);
    assert_eq!(replaced.code, 0, "{}", replaced.stderr);
    assert!(sandbox
        .run(&["prompt", "show"], None)
        .stdout
        .contains("proofreader"));
}

#[test]
fn prompt_use_with_an_unknown_name_lists_the_choices() {
    let out = Sandbox::new().run(&["prompt", "use", "shakespeare"], None);
    assert_eq!(out.code, 2);
    assert!(
        out.stderr.contains("typos") && out.stderr.contains("friendly"),
        "{}",
        out.stderr
    );
}
