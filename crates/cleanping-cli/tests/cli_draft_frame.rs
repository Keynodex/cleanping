//! The draft goes to the provider inside `<draft>` tags with a fixed sentence after the saved
//! prompt, but everything on this machine (history, `--keep-shape`, `prompt show`) keeps using
//! the text and prompt exactly as the user gave them.

mod support;

use cleanping_core::application::ports::RunRepository;
use cleanping_core::domain::draft_frame::{frame_draft, DRAFT_INSTRUCTIONS};
use cleanping_core::domain::shape::keeps_shape;
use cleanping_core::infrastructure::sqlite_db::Database;
use cleanping_core::infrastructure::sqlite_repositories::SqliteRunRepository;
use support::*;

fn with_server(response: String) -> (Sandbox, FakeServer) {
    let server = serve(vec![response]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    (sandbox, server)
}

#[test]
fn the_provider_gets_the_framed_draft_and_the_history_the_original() {
    let (sandbox, server) = with_server(ok_reply("<draft>\nPlease fix this.\n</draft>"));
    assert_eq!(sandbox.run(&["prompt", "set", "Be terse."], None).code, 0);
    let out = sandbox.run(&["pleae fix this"], None);
    assert_eq!(
        (out.code, out.stdout.as_str()),
        (0, "Please fix this.\n"),
        "{}",
        out.stderr
    );
    assert_eq!(server.raw_user_text(0), "<draft>\npleae fix this\n</draft>");
    assert_eq!(
        server.system_text(0),
        format!("Be terse.\n\n{DRAFT_INSTRUCTIONS}")
    );
    let runs = SqliteRunRepository::new(Database::new(sandbox.db_path()))
        .recent(5)
        .unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].input_text, "pleae fix this");
    assert_eq!(runs[0].prompt_text, "Be terse.");
    assert_eq!(runs[0].output_text.as_deref(), Some("Please fix this."));
    let shown = sandbox.run(&["prompt", "show"], None);
    assert_eq!(shown.stdout.trim_end(), "Be terse.");
}

#[test]
fn keep_shape_measures_the_reply_against_the_text_not_the_framed_draft() {
    // Two lines fit the three-line framed draft, but not the one-line text.
    let framed = frame_draft("fix").unwrap();
    assert!(
        keeps_shape(&framed, "line one\nline two"),
        "positive control"
    );
    assert!(!keeps_shape("fix", "line one\nline two"));
    let (sandbox, _server) = with_server(ok_reply("line one\nline two"));
    let out = sandbox.run(&["--keep-shape", "fix"], None);
    assert_eq!(out.code, 1, "the reply was used: {}", out.stdout);
    assert_eq!(out.stdout, "");
}

#[test]
fn keep_shape_accepts_a_one_line_reply_once_the_echoed_tags_are_gone() {
    let (sandbox, _server) = with_server(ok_reply("<draft>\nPlease fix this.\n</draft>"));
    let out = sandbox.run(&["--keep-shape", "pleae fix this"], None);
    assert_eq!(
        (out.code, out.stdout.as_str()),
        (0, "Please fix this.\n"),
        "{}",
        out.stderr
    );
}
