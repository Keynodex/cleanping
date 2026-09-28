//! Which key is used, how keys are named, and how a key is typed in.

mod support;

use support::*;

const NOWHERE: &str = "http://127.0.0.1:9/v1/chat/completions";

#[test]
fn an_exact_name_wins_and_a_case_variant_is_never_guessed() {
    let sandbox = Sandbox::new();
    sandbox.insert_credential("Foo", NOWHERE);
    sandbox.insert_credential("foo", NOWHERE);
    sandbox.store_secret("Foo", "sk-upper");
    sandbox.store_secret("foo", "sk-lower");
    let vague = sandbox.run(&["keys", "remove", "FOO"], None);
    assert_eq!(vague.code, 2, "{}", vague.stderr);
    assert!(vague.stderr.contains("exact name"), "{}", vague.stderr);
    assert_eq!(
        sandbox.run(&["keys", "list"], None).stdout.lines().count(),
        2
    );
    assert!(sandbox.has_secret("Foo") && sandbox.has_secret("foo"));
    assert_eq!(sandbox.run(&["keys", "remove", "foo"], None).code, 0);
    let left = sandbox.run(&["keys", "list"], None).stdout;
    assert!(left.contains("Foo") && !left.contains("foo"), "{left}");
    assert!(
        sandbox.has_secret("Foo"),
        "removing foo must not touch the key of Foo"
    );
    assert!(!sandbox.has_secret("foo"));
}

#[test]
fn either_of_two_names_that_differ_only_by_case_can_still_be_updated() {
    let sandbox = Sandbox::new();
    sandbox.insert_credential("Foo", NOWHERE);
    sandbox.insert_credential("foo", NOWHERE);
    let update = |name: &str| {
        sandbox.run(
            &[
                "keys", "add", "--name", name, "--url", NOWHERE, "--model", "newer",
            ],
            None,
        )
    };
    assert_eq!(update("Foo").code, 0);
    assert_eq!(update("foo").code, 0);
    let listed = sandbox.run(&["keys", "list"], None).stdout;
    assert_eq!(listed.matches("newer").count(), 2, "{listed}");
    assert_eq!(update("FOO").code, 2, "a third spelling is still refused");
}

#[test]
fn the_credential_flag_prefers_the_exact_name_over_a_case_variant() {
    let upper = serve(vec![ok_reply("from Foo")]);
    let lower = serve(vec![ok_reply("from foo")]);
    let sandbox = Sandbox::new();
    sandbox.insert_credential("Foo", &upper.url);
    sandbox.insert_credential("foo", &lower.url);
    assert_eq!(sandbox.run(&["-c", "foo", "x"], None).stdout, "from foo\n");
    assert_eq!(sandbox.run(&["-c", "Foo", "x"], None).stdout, "from Foo\n");
    let vague = sandbox.run(&["-c", "FOO", "x"], None);
    assert_eq!(vague.code, 2, "{}", vague.stderr);
    assert_eq!(upper.requests.lock().unwrap().len(), 1);
    assert_eq!(lower.requests.lock().unwrap().len(), 1);
}

#[test]
fn several_keys_and_none_selected_never_sends_the_text_anywhere() {
    let server = serve(vec![ok_reply("nope")]);
    let sandbox = Sandbox::new();
    sandbox.insert_credential("Alpha", &server.url);
    sandbox.insert_credential("Beta", &server.url);
    let out = sandbox.run(&["hello"], None);
    assert_eq!((out.code, out.stdout.as_str()), (3, ""));
    assert!(out.stderr.contains("none is selected"), "{}", out.stderr);
    assert!(out.stderr.contains("cleanping keys use"), "{}", out.stderr);
    assert!(server.requests.lock().unwrap().is_empty());
}

#[test]
fn a_single_key_is_used_even_when_none_is_selected() {
    let server = serve(vec![ok_reply("fine")]);
    let sandbox = Sandbox::new();
    sandbox.insert_credential("Only", &server.url);
    let out = sandbox.run(&["hello"], None);
    assert_eq!((out.code, out.stdout.as_str()), (0, "fine\n"));
}

#[test]
fn the_missing_key_hint_is_quoted_for_the_shell_and_does_not_suggest_echoing() {
    let sandbox = Sandbox::new();
    sandbox.insert_credential("Bob's key", "https://api.example.com/v1/chat/completions");
    let out = sandbox.run(&["hello"], None);
    assert_eq!(out.code, 3, "{}", out.stderr);
    assert!(
        out.stderr.contains(
            "cleanping keys add --name 'Bob'\\''s key' \
             --url 'https://api.example.com/v1/chat/completions' --model 'test-model'"
        ),
        "{}",
        out.stderr
    );
    assert!(!out.stderr.contains("--key-stdin"), "{}", out.stderr);
}

#[test]
fn listing_into_a_closed_pipe_ends_quietly() {
    let sandbox = Sandbox::new();
    sandbox.insert_credential("Foo", NOWHERE);
    let out = sandbox.run_with_closed_stdout(&["keys", "list"]);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(!out.stderr.contains("panicked"), "{}", out.stderr);
}

// `/dev/full` (every write fails with "no space left") exists on Linux only.
#[cfg(target_os = "linux")]
#[test]
fn a_failing_stdout_is_reported_not_a_panic() {
    let sandbox = Sandbox::new();
    sandbox.insert_credential("Foo", NOWHERE);
    let out = sandbox.run_with_full_disk(&["keys", "list"]);
    assert_eq!(out.code, 1, "{}", out.stderr);
    assert!(out.stderr.starts_with("cleanping: "), "{}", out.stderr);
    assert!(!out.stderr.contains("panicked"), "{}", out.stderr);
}

#[test]
fn a_key_of_exactly_the_limit_is_accepted_even_with_a_windows_line_ending() {
    let sandbox = Sandbox::new();
    let add = |key: &str| {
        sandbox.run(
            &["keys", "add", "--provider", "OpenAI", "--key-stdin"],
            Some(&format!("{key}\r\n")),
        )
    };
    let fits = add(&"k".repeat(4096));
    assert_eq!(fits.code, 0, "{}", fits.stderr);
    let too_big = add(&"k".repeat(4097));
    assert_eq!(too_big.code, 2, "{}", too_big.stderr);
    assert!(too_big.stderr.contains("limit 4096"), "{}", too_big.stderr);
}

#[cfg(target_os = "linux")]
#[test]
fn a_key_typed_at_a_terminal_is_never_echoed_even_with_key_stdin() {
    use std::time::Duration;

    let sandbox = Sandbox::new();
    let env = sandbox.environment(Some(&sandbox.path_with_binary()), &[]);
    let mut terminal = pty::Pty::start(&env, "cleanping keys add --provider OpenAI --key-stdin");
    assert!(
        terminal.wait_for_screen("API key", Duration::from_secs(10)),
        "no prompt: {}",
        terminal.screen()
    );
    // The prompt is printed just before the terminal is switched to no-echo.
    std::thread::sleep(Duration::from_millis(300));
    terminal.send("sk-typed-at-the-terminal\n");
    assert!(
        terminal.wait_for_screen("Saved", Duration::from_secs(10)),
        "not saved: {}",
        terminal.screen()
    );
    let screen = terminal.screen();
    assert_eq!(terminal.finish(Duration::from_secs(10)), 0);
    assert!(!screen.contains("sk-typed-at-the-terminal"), "{screen}");
    assert!(sandbox
        .run(&["keys", "list"], None)
        .stdout
        .contains("key saved"));
}
