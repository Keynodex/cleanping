//! Looking at and deleting the local history.

mod support;

use cleanping_core::infrastructure::sqlite_db::Database;
use support::*;

fn two_rewrites() -> Sandbox {
    let server = serve(vec![ok_reply("First."), ok_reply("Second.")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    sandbox.run(&["first  rough\ttext"], None);
    sandbox.run(&["second rough text"], None);
    sandbox
}

#[test]
fn list_shows_the_newest_first_on_one_line_each() {
    let sandbox = two_rewrites();
    let out = sandbox.run(&["history", "list"], None);
    assert_eq!(out.code, 0, "{}", out.stderr);
    let lines: Vec<&str> = out.stdout.lines().collect();
    assert_eq!(lines.len(), 2, "{}", out.stdout);
    assert!(lines[0].ends_with("second rough text"), "{}", lines[0]);
    assert!(lines[1].ends_with("first rough text"), "{}", lines[1]);
    assert!(lines[0].contains(" ok ") && lines[0].contains("test-model"));
    let limited = sandbox.run(&["history", "list", "--limit", "1"], None);
    assert_eq!(limited.stdout.lines().count(), 1);
}

#[test]
fn a_long_input_is_shortened_in_the_list() {
    let server = serve(vec![ok_reply("ok")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    sandbox.run(&[&"word ".repeat(100)], None);
    let out = sandbox.run(&["history", "list"], None);
    let line = out.stdout.lines().next().unwrap();
    assert!(line.ends_with('\u{2026}') && line.len() < 200, "{line}");
}

#[test]
fn an_empty_history_lists_nothing_and_succeeds() {
    let out = Sandbox::new().run(&["history", "list"], None);
    assert_eq!((out.code, out.stdout.as_str()), (0, ""));
}

#[test]
fn clear_needs_an_explicit_yes() {
    let sandbox = two_rewrites();
    let refused = sandbox.run(&["history", "clear"], None);
    assert_eq!(refused.code, 2, "{}", refused.stderr);
    assert!(refused.stderr.contains("--yes"), "{}", refused.stderr);
    assert_eq!(
        sandbox
            .run(&["history", "list"], None)
            .stdout
            .lines()
            .count(),
        2
    );
    let done = sandbox.run(&["history", "clear", "--yes"], None);
    assert_eq!((done.code, done.stdout.as_str()), (0, "Deleted 2 runs.\n"));
    assert_eq!(sandbox.run(&["history", "list"], None).stdout, "");
    let again = sandbox.run(&["history", "clear", "--yes"], None);
    assert_eq!(again.stdout, "Deleted 0 runs.\n");
}

#[test]
fn purge_removes_only_runs_older_than_the_given_days() {
    let sandbox = two_rewrites();
    Database::new(sandbox.db_path())
        .with(|c| {
            c.execute(
                "INSERT INTO runs (created_at, model, prompt_text, input_text, status, duration_ms)
                 VALUES ('2020-01-01T00:00:00+00:00', 'm', 'p', 'ancient', 'ok', 1)",
                [],
            )
        })
        .unwrap();
    let out = sandbox.run(&["history", "purge", "--older-than", "30"], None);
    assert_eq!((out.code, out.stdout.as_str()), (0, "Deleted 1 run.\n"));
    let left = sandbox.run(&["history", "list"], None).stdout;
    assert_eq!(left.lines().count(), 2);
    assert!(!left.contains("ancient"));
}

#[test]
fn no_field_can_send_control_characters_to_the_terminal() {
    let sandbox = two_rewrites();
    Database::new(sandbox.db_path())
        .with(|c| {
            c.execute(
                "INSERT INTO runs (created_at, model, prompt_text, input_text, status, duration_ms)
                 VALUES ('2026-01-01T00:00:0\u{1b}[2J+00:00', 'evil\u{1b}]0;title\u{7}model', 'p',
                         'text\u{1b}[31m', 'ok', 1)",
                [],
            )
        })
        .unwrap();
    let out = sandbox.run(&["history", "list"], None);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert_eq!(out.stdout.lines().count(), 3, "{:?}", out.stdout);
    assert!(!out.stdout.contains('\u{1b}'), "{:?}", out.stdout);
    assert!(!out.stdout.contains('\u{7}'), "{:?}", out.stdout);
}

#[test]
fn purge_needs_a_sensible_number_of_days() {
    let sandbox = two_rewrites();
    for bad in ["0", "abc", "-3", "99999999"] {
        let out = sandbox.run(&["history", "purge", "--older-than", bad], None);
        assert_eq!(out.code, 2, "{bad}: {}", out.stderr);
    }
    assert_eq!(sandbox.run(&["history", "purge"], None).code, 2);
    assert_eq!(
        sandbox
            .run(&["history", "list"], None)
            .stdout
            .lines()
            .count(),
        2
    );
}
