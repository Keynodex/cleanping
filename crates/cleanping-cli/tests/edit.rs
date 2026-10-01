//! `cleanping edit FILE`: the screen a host app (Claude Code, Codex) shows when it opens
//! `$VISUAL`. Each test drives the real binary inside a pseudo-terminal (Linux `script`) and
//! waits for what it prints, never for a fixed time.
#![cfg(target_os = "linux")]

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use support::pty::Pty;
use support::*;

const ENTER: &str = "\r";
const ESCAPE: &str = "\x1b";
const LIMIT: Duration = Duration::from_secs(20);
const ROUGH: &str = "pls fix teh login\n";
const CLEAN: &str = "Please fix the login.";
/// Looks like a real API key to the secret check, and is not one. Written in two pieces so no
/// line in this file looks like a real key to a secret scanner.
const FAKE_KEY: &str = concat!("sk-", "Zx81QmVt3LpRw92NcYb47HdKe06Fa5Ug");

struct Setup {
    sandbox: Sandbox,
    server: FakeServer,
    file: PathBuf,
}

fn setup(replies: Vec<String>, text: &str) -> Setup {
    let sandbox = Sandbox::new();
    let server = serve(replies);
    sandbox.add_local("local", &server);
    let file = sandbox.dir.path().join("prompt.txt");
    std::fs::write(&file, text).unwrap();
    Setup {
        sandbox,
        server,
        file,
    }
}

fn quoted(path: &Path) -> String {
    format!("'{}'", path.display())
}

/// `command` runs in a pseudo-terminal 80 columns wide with the built binary on its PATH.
fn terminal(sandbox: &Sandbox, command: &str) -> Pty {
    let env = sandbox.environment(Some(&sandbox.path_with_binary()), &[("TERM", "xterm")]);
    Pty::start(&env, &format!("stty rows 24 cols 80; {command}"))
}

fn edit(setup: &Setup) -> Pty {
    terminal(
        &setup.sandbox,
        &format!("cleanping edit {}", quoted(&setup.file)),
    )
}

fn contents(setup: &Setup) -> String {
    std::fs::read_to_string(&setup.file).unwrap()
}

fn request_count(setup: &Setup) -> usize {
    setup.server.requests.lock().unwrap().len()
}

#[test]
fn enter_accepts_the_edited_text() {
    let s = setup(vec![ok_reply(CLEAN)], ROUGH);
    let mut screen = edit(&s);
    assert!(
        screen.wait_for_text("Text edit complete", LIMIT),
        "{}",
        screen.plain_screen()
    );
    // The footer is drawn after the header, in later writes, so wait for it too.
    assert!(
        screen.wait_for_text("Enter accept", LIMIT),
        "{}",
        screen.plain_screen()
    );
    screen.send(ENTER);
    assert_eq!(screen.finish(LIMIT), 0);
    assert_eq!(contents(&s), format!("{CLEAN}\n"));
    assert_eq!(s.server.user_text(0), "pls fix teh login");
}

#[test]
fn n_keeps_the_original_text() {
    let s = setup(vec![ok_reply(CLEAN)], ROUGH);
    let mut screen = edit(&s);
    assert!(screen.wait_for_text("Text edit complete", LIMIT));
    screen.send("n");
    assert_eq!(screen.finish(LIMIT), 0);
    assert_eq!(contents(&s), ROUGH);
}

#[test]
fn o_shows_the_original_and_back() {
    let s = setup(vec![ok_reply(CLEAN)], ROUGH);
    let mut screen = edit(&s);
    assert!(screen.wait_for_text("Text edit complete", LIMIT));
    screen.send("o");
    assert!(
        screen.wait_for_text("Original text (not edited)", LIMIT),
        "{}",
        screen.plain_screen()
    );
    screen.send("o");
    screen.send(ENTER);
    assert_eq!(screen.finish(LIMIT), 0);
    assert_eq!(contents(&s), format!("{CLEAN}\n"));
}

#[test]
fn ctrl_c_cancels_and_the_terminal_is_put_back() {
    let s = setup(vec![ok_reply(CLEAN)], ROUGH);
    let mut screen = edit(&s);
    assert!(screen.wait_for_text("Text edit complete", LIMIT));
    assert!(screen.screen().contains("\x1b[?1049h"), "alternate screen");
    screen.send("\x03");
    let restored = || {
        let raw = screen.screen();
        raw.contains("\x1b[?1049l") && raw.contains("\x1b[?25h")
    };
    assert!(screen.wait_until(LIMIT, restored), "{:?}", screen.screen());
    assert_eq!(screen.finish(LIMIT), 0);
    assert_eq!(contents(&s), ROUGH);
}

#[test]
fn the_edit_is_not_saved_to_the_history() {
    let s = setup(vec![ok_reply(CLEAN), ok_reply(CLEAN)], ROUGH);
    let mut screen = edit(&s);
    assert!(screen.wait_for_text("Text edit complete", LIMIT));
    screen.send(ENTER);
    assert_eq!(screen.finish(LIMIT), 0);
    let history = s.sandbox.run(&["history", "list"], None);
    assert_eq!(history.code, 0);
    assert!(!history.stdout.contains("pls fix"), "{}", history.stdout);
    // The same list does show an ordinary rewrite, so the empty result above means something.
    assert_eq!(s.sandbox.run(&["pls fix teh login"], None).code, 0);
    let history = s.sandbox.run(&["history", "list"], None);
    assert!(history.stdout.contains("pls fix"), "{}", history.stdout);
}

#[test]
fn a_failure_is_shown_and_the_text_is_left_alone() {
    let s = setup(vec![status_reply(500, "provider-said-boom")], ROUGH);
    let mut screen = edit(&s);
    assert!(
        screen.wait_for_text("Could not edit", LIMIT),
        "{}",
        screen.plain_screen()
    );
    // Wait for the footer, the last line drawn, so the check below covers the whole screen.
    assert!(
        screen.wait_for_text("Press any key to keep your text as it is", LIMIT),
        "{}",
        screen.plain_screen()
    );
    assert!(!screen.plain_screen().contains("provider-said-boom"));
    screen.send("x");
    assert_eq!(screen.finish(LIMIT), 0);
    assert_eq!(contents(&s), ROUGH);
}

#[test]
fn no_saved_key_is_explained_on_the_screen() {
    let sandbox = Sandbox::new();
    let file = sandbox.dir.path().join("prompt.txt");
    std::fs::write(&file, ROUGH).unwrap();
    let mut screen = terminal(&sandbox, &format!("cleanping edit {}", quoted(&file)));
    assert!(
        screen.wait_for_text("keys add", LIMIT),
        "{}",
        screen.plain_screen()
    );
    screen.send("x");
    assert_eq!(screen.finish(LIMIT), 0);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), ROUGH);
}

#[test]
fn text_that_looks_like_a_secret_is_not_sent() {
    let text = format!("my key is {FAKE_KEY} please help\n");
    let s = setup(vec![ok_reply(CLEAN)], &text);
    let mut screen = edit(&s);
    assert!(
        screen.wait_for_text("Not sent", LIMIT),
        "{}",
        screen.plain_screen()
    );
    // The screen arrives in many small writes: the header first, then the body, then the
    // footer, the last line drawn. Wait for each text before relying on it.
    assert!(
        screen.wait_for_text("an API key or token", LIMIT),
        "{}",
        screen.plain_screen()
    );
    assert!(
        screen.wait_for_text("any other key keeps your text", LIMIT),
        "{}",
        screen.plain_screen()
    );
    assert!(!screen.plain_screen().contains(FAKE_KEY));
    screen.send("x");
    assert_eq!(screen.finish(LIMIT), 0);
    assert_eq!(request_count(&s), 0);
    assert_eq!(contents(&s), text);
}

#[test]
fn s_sends_secret_looking_text_after_a_deliberate_choice() {
    let text = format!("my key is {FAKE_KEY} please help\n");
    let s = setup(vec![ok_reply(CLEAN)], &text);
    let mut screen = edit(&s);
    assert!(screen.wait_for_text("Not sent", LIMIT));
    screen.send("s");
    assert!(screen.wait_for_text("Text edit complete", LIMIT));
    screen.send(ENTER);
    assert_eq!(screen.finish(LIMIT), 0);
    assert_eq!(request_count(&s), 1);
    assert_eq!(contents(&s), format!("{CLEAN}\n"));
}

#[test]
fn escape_while_waiting_leaves_at_once() {
    let (server, release) = serve_held(ok_reply(CLEAN));
    let sandbox = Sandbox::new();
    sandbox.add_local("local", &server);
    let file = sandbox.dir.path().join("prompt.txt");
    std::fs::write(&file, ROUGH).unwrap();
    let mut screen = terminal(&sandbox, &format!("cleanping edit {}", quoted(&file)));
    assert!(
        screen.wait_for_text("editing with", LIMIT),
        "{}",
        screen.plain_screen()
    );
    screen.send(ESCAPE);
    assert_eq!(screen.finish(LIMIT), 0);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), ROUGH);
    let _ = release.send(());
}

#[test]
fn it_works_as_visual_with_stdout_redirected() {
    // A host app runs `$VISUAL FILE` with stdout not on the terminal; the screen uses /dev/tty.
    let s = setup(vec![ok_reply(CLEAN)], ROUGH);
    let mut screen = terminal(
        &s.sandbox,
        &format!(
            "VISUAL='cleanping edit'; sh -c \"$VISUAL {}\" > /dev/null",
            quoted(&s.file)
        ),
    );
    assert!(
        screen.wait_for_text("Text edit complete", LIMIT),
        "{}",
        screen.plain_screen()
    );
    screen.send(ENTER);
    assert_eq!(screen.finish(LIMIT), 0);
    assert_eq!(contents(&s), format!("{CLEAN}\n"));
}

#[test]
fn a_long_text_can_be_scrolled() {
    let text: String = (1..=60).map(|n| format!("line number {n}\n")).collect();
    let s = setup(vec![ok_reply(&text)], &text);
    let mut screen = edit(&s);
    assert!(screen.wait_for_text("Text edit complete", LIMIT));
    assert!(
        screen.wait_for_text("scroll", LIMIT),
        "{}",
        screen.plain_screen()
    );
    screen.send("\x1b[6~"); // Page Down
    assert!(
        screen.wait_for_text("line number 30", LIMIT),
        "{}",
        screen.plain_screen()
    );
    screen.send("n");
    assert_eq!(screen.finish(LIMIT), 0);
}

// Without a screen (no terminal at all): `--yes` edits in place, nothing else may.

fn without_terminal(setup: &Setup, args: &[&str]) -> Out {
    let binary = env!("CARGO_BIN_EXE_cleanping");
    let output = Command::new("setsid")
        .arg("-w")
        .arg(binary)
        .args(args)
        .arg(&setup.file)
        .env_clear()
        .envs(setup.sandbox.environment(None, &[]))
        .stdin(std::process::Stdio::null())
        .output()
        .expect("setsid is needed for terminal tests");
    Out {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

#[test]
fn yes_edits_the_file_without_a_screen() {
    let s = setup(vec![ok_reply(CLEAN)], ROUGH);
    let out = without_terminal(&s, &["edit", "--yes"]);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert_eq!(out.stdout, "");
    assert_eq!(contents(&s), format!("{CLEAN}\n"));
}

#[test]
fn yes_never_sends_secret_looking_text() {
    let text = format!("token {FAKE_KEY}\n");
    let s = setup(vec![ok_reply(CLEAN)], &text);
    let out = without_terminal(&s, &["edit", "--yes"]);
    assert_eq!(out.code, 2);
    assert!(out.stderr.contains("an API key or token"), "{}", out.stderr);
    assert!(!out.stderr.contains(FAKE_KEY));
    assert_eq!(request_count(&s), 0);
    assert_eq!(contents(&s), text);
}

#[test]
fn yes_reports_a_failure_and_keeps_the_text() {
    let s = setup(vec![status_reply(500, "provider-said-boom")], ROUGH);
    let out = without_terminal(&s, &["edit", "--yes"]);
    assert_eq!(out.code, 1);
    assert!(!out.stderr.contains("provider-said-boom"));
    assert_eq!(contents(&s), ROUGH);
}

#[test]
fn without_a_terminal_or_yes_it_says_what_to_do() {
    let s = setup(vec![ok_reply(CLEAN)], ROUGH);
    let out = without_terminal(&s, &["edit"]);
    assert_eq!(out.code, 1);
    assert!(out.stderr.contains("--yes"), "{}", out.stderr);
    assert_eq!(request_count(&s), 0);
    assert_eq!(contents(&s), ROUGH);
}

#[test]
fn a_blank_file_is_left_alone_and_nothing_is_sent() {
    let s = setup(vec![ok_reply(CLEAN)], "  \n");
    let out = without_terminal(&s, &["edit"]);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert_eq!(request_count(&s), 0);
    assert_eq!(contents(&s), "  \n");
}

#[test]
fn a_missing_file_is_a_bad_input() {
    let s = setup(vec![], ROUGH);
    std::fs::remove_file(&s.file).unwrap();
    let out = without_terminal(&s, &["edit", "--yes"]);
    assert_eq!(out.code, 2);
}
