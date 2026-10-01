//! A reply the provider cut off at its output limit (`finish_reason: "length"`) is refused:
//! half a rewrite that looks finished must never replace the user's text.

mod support;

use cleanping_core::application::ports::RunRepository;
use cleanping_core::domain::models::RunStatus;
use cleanping_core::infrastructure::sqlite_db::Database;
use cleanping_core::infrastructure::sqlite_repositories::SqliteRunRepository;
use support::*;

const TYPED: &str = "pleae fix this long text";
const HALF: &str = "Please fix this lo";

fn with_server(response: String) -> (Sandbox, FakeServer) {
    let server = serve(vec![response]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    (sandbox, server)
}

fn assert_refused(out: &Out) {
    assert_eq!(out.code, 1, "the cut-off reply was used: {}", out.stderr);
    assert_eq!(out.stdout, "", "nothing of the reply may reach stdout");
    assert!(
        out.stderr
            .contains("The reply was cut off at the provider's length limit"),
        "{}",
        out.stderr
    );
    assert!(out.stderr.contains("shorter text"), "{}", out.stderr);
    assert!(
        !out.stderr.contains(HALF),
        "the reply leaked: {}",
        out.stderr
    );
}

#[test]
fn a_plain_rewrite_refuses_a_cut_off_reply() {
    let (sandbox, server) = with_server(cut_off_reply(HALF));
    let out = sandbox.run(&[TYPED], None);
    assert_refused(&out);
    assert_eq!(server.requests.lock().unwrap().len(), 1);
}

#[test]
fn the_history_records_a_failure_and_none_of_the_reply() {
    let (sandbox, _server) = with_server(cut_off_reply(HALF));
    let out = sandbox.run(&[TYPED], None);
    assert_eq!(out.code, 1, "the cut-off reply was used: {}", out.stderr);
    let runs = SqliteRunRepository::new(Database::new(sandbox.db_path()))
        .recent(5)
        .unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(
        runs[0].status,
        RunStatus::Error,
        "the cut-off reply was used"
    );
    assert_eq!(runs[0].output_text, None);
    let reason = runs[0].error_message.clone().unwrap_or_default();
    assert!(reason.contains("cut off"), "{reason}");
}

#[test]
fn the_shell_key_flags_refuse_a_cut_off_reply() {
    // The same flags the shell key passes; stdout is what would replace the typed line.
    let (sandbox, _server) = with_server(cut_off_reply(HALF));
    let out = sandbox.run(
        &["--no-history", "--keep-shape", "--refuse-secrets"],
        Some(TYPED),
    );
    assert_refused(&out);
}

#[test]
fn a_reply_that_finished_normally_is_still_used() {
    let (sandbox, _server) = with_server(finished_reply("Please fix this.", "stop"));
    let out = sandbox.run(&[TYPED], None);
    assert_eq!(
        (out.code, out.stdout.as_str()),
        (0, "Please fix this.\n"),
        "{}",
        out.stderr
    );
}

#[test]
fn keys_test_still_passes_when_the_provider_finishes() {
    let (sandbox, _server) = with_server(finished_reply("OK", "stop"));
    let out = sandbox.run(&["keys", "test"], None);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(out.stdout.starts_with("OK: "), "{}", out.stdout);
}

#[test]
fn keys_test_names_the_key_when_even_the_test_reply_is_cut_off() {
    // A model that cannot answer "OK" within its limit cannot finish a rewrite either.
    let (sandbox, _server) = with_server(cut_off_reply(""));
    let out = sandbox.run(&["keys", "test"], None);
    assert_eq!(out.code, 1, "{}", out.stderr);
    assert_eq!(out.stdout, "");
    assert!(
        out.stderr.contains("\u{201c}Local\u{201d}") && out.stderr.contains("cut off"),
        "the cut-off reply was used: {}",
        out.stderr
    );
}
