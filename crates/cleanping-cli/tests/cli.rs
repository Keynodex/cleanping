//! End-to-end behavior of the `cleanping` binary against a fake local API.

mod support;

use cleanping_core::application::ports::RunRepository;
use cleanping_core::infrastructure::sqlite_db::Database;
use cleanping_core::infrastructure::sqlite_repositories::SqliteRunRepository;
use support::*;

fn history_len(sandbox: &Sandbox) -> usize {
    SqliteRunRepository::new(Database::new(sandbox.db_path()))
        .recent(50)
        .unwrap()
        .len()
}

#[test]
fn version_flag_prints_the_name_and_version() {
    let out = Sandbox::new().run(&["--version"], None);
    assert_eq!(out.code, 0);
    assert!(out.stdout.starts_with("cleanping "), "{}", out.stdout);
}

#[test]
fn an_argument_is_rewritten_and_only_the_text_goes_to_stdout() {
    let server = serve(vec![ok_reply("Please fix this.")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let out = sandbox.run(&["pleae", "fix", "this"], None);
    assert_eq!(
        (out.code, out.stdout.as_str(), out.stderr.as_str()),
        (0, "Please fix this.\n", "")
    );
    assert_eq!(server.user_text(0), "pleae fix this");
}

#[test]
fn piped_stdin_is_used_when_there_is_no_argument() {
    let server = serve(vec![ok_reply("Clean text.")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let out = sandbox.run(&[], Some("  rough\ntext  \n"));
    assert_eq!((out.code, out.stdout.as_str()), (0, "Clean text.\n"));
    assert_eq!(server.user_text(0), "rough\ntext");
}

#[test]
fn empty_input_is_a_usage_error() {
    let server = serve(vec![]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let out = sandbox.run(&[], Some("   \n"));
    assert_eq!(out.code, 2);
    assert!(out.stderr.contains("Nothing to rewrite"), "{}", out.stderr);
    assert_eq!(out.stdout, "");
}

#[test]
fn oversized_input_is_refused_before_any_request() {
    let server = serve(vec![]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let out = sandbox.run(&[], Some(&"x".repeat(300_000)));
    assert_eq!(out.code, 2);
    assert!(out.stderr.contains("too long"), "{}", out.stderr);
    assert!(server.requests.lock().unwrap().is_empty());
}

#[test]
fn no_saved_credential_exits_3_with_a_hint() {
    let out = Sandbox::new().run(&["hello"], None);
    assert_eq!(out.code, 3);
    assert!(out.stderr.contains("cleanping keys add"), "{}", out.stderr);
    assert_eq!(out.stdout, "");
}

#[test]
fn a_provider_error_exits_1_with_an_empty_stdout() {
    let server = serve(vec![status_reply(401, "secret server detail")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let out = sandbox.run(&["hello"], None);
    assert_eq!((out.code, out.stdout.as_str()), (1, ""));
    assert!(out.stderr.contains("HTTP 401"));
    assert!(!out.stderr.contains("secret server detail"));
}

#[test]
fn history_is_recorded_unless_no_history_is_given() {
    let server = serve(vec![ok_reply("a"), ok_reply("b")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    sandbox.run(&["--no-history", "one"], None);
    assert_eq!(history_len(&sandbox), 0);
    sandbox.run(&["two"], None);
    assert_eq!(history_len(&sandbox), 1);
}

#[test]
fn keys_add_list_remove_and_the_secret_is_never_printed() {
    let server = serve(vec![]);
    let sandbox = Sandbox::new();
    let added = sandbox.run(
        &[
            "keys",
            "add",
            "--name",
            "Work",
            "--url",
            &server.url,
            "--model",
            "m",
            "--key-stdin",
        ],
        Some("sk-super-secret\n"),
    );
    assert_eq!(added.code, 0, "{}", added.stderr);
    let listed = sandbox.run(&["keys", "list"], None);
    assert!(listed.stdout.contains("Work") && listed.stdout.contains("m"));
    for text in [&added.stdout, &added.stderr, &listed.stdout, &listed.stderr] {
        assert!(!text.contains("sk-super-secret"));
    }
    assert_eq!(sandbox.run(&["keys", "remove", "Work"], None).code, 0);
    assert!(!sandbox.run(&["keys", "list"], None).stdout.contains("Work"));
}

#[test]
fn a_remote_http_url_is_rejected_with_exit_2() {
    let out = Sandbox::new().run(
        &[
            "keys",
            "add",
            "--name",
            "Bad",
            "--url",
            "http://remote.example.com/v1",
            "--model",
            "m",
            "--key-stdin",
        ],
        Some("sk-x\n"),
    );
    assert_eq!(out.code, 2);
    assert!(out.stderr.contains("HTTPS"));
}

#[test]
fn a_provider_preset_fills_url_and_model_and_needs_a_key() {
    let sandbox = Sandbox::new();
    let missing = sandbox.run(&["keys", "add", "--provider", "OpenAI"], None);
    assert_eq!(missing.code, 2);
    assert!(missing.stderr.contains("API key"), "{}", missing.stderr);
    let ok = sandbox.run(
        &["keys", "add", "--provider", "OpenAI", "--key-stdin"],
        Some("sk-x\n"),
    );
    assert_eq!(ok.code, 0, "{}", ok.stderr);
    assert!(sandbox
        .run(&["keys", "list"], None)
        .stdout
        .contains("gpt-4o-mini"));
}

#[test]
fn the_credential_flag_picks_by_name_and_use_changes_the_default() {
    let one = serve(vec![ok_reply("from one")]);
    let two = serve(vec![ok_reply("from two"), ok_reply("from two again")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Alpha", &one);
    sandbox.add_local("Beta", &two);
    assert_eq!(sandbox.run(&["-c", "beta", "x"], None).stdout, "from two\n");
    assert_eq!(sandbox.run(&["keys", "use", "Beta"], None).code, 0);
    assert_eq!(sandbox.run(&["x"], None).stdout, "from two again\n");
    assert_eq!(sandbox.run(&["-c", "Nope", "x"], None).code, 3);
}

#[test]
fn the_system_prompt_can_be_shown_and_changed() {
    let server = serve(vec![ok_reply("ok")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    assert!(sandbox
        .run(&["prompt", "show"], None)
        .stdout
        .contains("precise copy editor"));
    assert_eq!(sandbox.run(&["prompt", "set", "Be terse."], None).code, 0);
    sandbox.run(&["hello"], None);
    assert_eq!(server.system_text(0), "Be terse.");
}

#[test]
fn copy_without_a_clipboard_tool_warns_but_still_prints_the_text() {
    let server = serve(vec![ok_reply("Done.")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let out = sandbox.run_with_path(&["--copy", "hi"], None, Some("/nonexistent"));
    assert_eq!((out.code, out.stdout.as_str()), (0, "Done.\n"));
    assert!(out.stderr.contains("clipboard"), "{}", out.stderr);
}

#[test]
fn an_absurdly_long_key_on_stdin_is_refused() {
    let sandbox = Sandbox::new();
    let out = sandbox.run(
        &["keys", "add", "--provider", "OpenAI", "--key-stdin"],
        Some(&format!("{}\n", "k".repeat(10_000))),
    );
    assert_eq!(out.code, 2);
    assert!(out.stderr.contains("too long"), "{}", out.stderr);
}
