//! Typing an API key in a terminal: shown as stars, editable, and Ctrl-C never leaves the
//! terminal with typing switched off. Drives the real binary in a pseudo-terminal (Linux `script`).
#![cfg(target_os = "linux")]

mod support;

use std::time::Duration;

use support::pty::Pty;
use support::*;

const LIMIT: Duration = Duration::from_secs(20);
/// After a prompt appears the terminal needs a moment to change its mode.
const SETTLE: Duration = Duration::from_millis(150);
/// Not a real key.
const SECRET: &str = "hidden-key-not-real-777";
const PROMPT: &str = "API key (hidden; Enter to skip for a local model)";

/// `command` runs in a terminal with the built binary on PATH.
fn terminal(sandbox: &Sandbox, command: &str) -> Pty {
    let env = sandbox.environment(Some(&sandbox.path_with_binary()), &[("TERM", "xterm")]);
    Pty::start(&env, command)
}

fn at_the_key_prompt(sandbox: &Sandbox, command: &str) -> Pty {
    let screen = terminal(sandbox, command);
    assert!(
        screen.wait_for_text(PROMPT, LIMIT),
        "waiting for the key prompt: {}",
        screen.plain_screen()
    );
    std::thread::sleep(SETTLE);
    screen
}

const ADD: &str = "cleanping keys add --provider OpenAI";

#[test]
fn a_typed_key_is_saved_and_shown_only_as_stars() {
    let sandbox = Sandbox::new();
    let mut screen = at_the_key_prompt(&sandbox, ADD);
    screen.send(SECRET);
    let stars = "*".repeat(SECRET.len());
    assert!(
        screen.wait_for_text(&stars, LIMIT),
        "{}",
        screen.plain_screen()
    );
    screen.send("\r");
    assert!(screen.wait_for_text("Saved", LIMIT));
    let everything = screen.screen();
    assert_eq!(screen.finish(LIMIT), 0);
    assert!(!everything.contains(SECRET), "the key was echoed");
    assert_eq!(sandbox.secret_value("OpenAI").as_deref(), Some(SECRET));
}

#[test]
fn backspace_and_ctrl_u_edit_what_is_typed() {
    let sandbox = Sandbox::new();
    let mut screen = at_the_key_prompt(&sandbox, ADD);
    screen.send("abcX\x7fd"); // abcX, Backspace, d
    screen.send("\x15"); // Ctrl-U clears the line
    screen.send("right-one\r");
    assert!(screen.wait_for_text("Saved", LIMIT));
    assert_eq!(screen.finish(LIMIT), 0);
    assert_eq!(sandbox.secret_value("OpenAI").as_deref(), Some("right-one"));
}

#[test]
fn a_line_feed_ends_the_key_like_enter_does() {
    let sandbox = Sandbox::new();
    let mut screen = at_the_key_prompt(&sandbox, ADD);
    screen.send("via-line-feed\n");
    assert!(screen.wait_for_text("Saved", LIMIT));
    assert_eq!(screen.finish(LIMIT), 0);
    assert_eq!(
        sandbox.secret_value("OpenAI").as_deref(),
        Some("via-line-feed")
    );
}

#[test]
fn enter_alone_skips_the_key_for_a_local_model() {
    let sandbox = Sandbox::new();
    let mut screen = at_the_key_prompt(&sandbox, "cleanping keys add --provider 'Ollama (local)'");
    screen.send("\r");
    assert!(
        screen.wait_for_text("Saved", LIMIT),
        "{}",
        screen.plain_screen()
    );
    assert_eq!(screen.finish(LIMIT), 0);
    assert!(!sandbox.has_secret("Ollama (local)"));
}

/// Ctrl-C at the prompt must end the command cleanly (not by a signal) and give the terminal
/// back with typing switched on. `stty -a` afterwards shows whether echo is on.
#[test]
fn ctrl_c_at_the_key_prompt_saves_nothing_and_restores_typing() {
    let sandbox = Sandbox::new();
    let mut screen = at_the_key_prompt(&sandbox, &format!("{ADD}; echo EXIT=$?; stty -a"));
    screen.send("half-typed\x03");
    assert!(
        screen.wait_for_text("EXIT=", LIMIT),
        "{}",
        screen.plain_screen()
    );
    assert!(
        screen.wait_for_text("speed", LIMIT),
        "{}",
        screen.plain_screen()
    );
    let text = screen.plain_screen();
    assert!(
        text.contains("EXIT=2"),
        "ended by our own message, not a signal: {text}"
    );
    assert!(
        text.contains(" echo ") && !text.contains("-echo "),
        "typing left switched off: {text}"
    );
    assert!(!sandbox.has_secret("OpenAI"));
    let _ = screen.finish(LIMIT);
}

#[test]
fn escape_also_cancels() {
    let sandbox = Sandbox::new();
    let mut screen = at_the_key_prompt(&sandbox, &format!("{ADD}; echo EXIT=$?"));
    screen.send("\x1b");
    assert!(
        screen.wait_for_text("EXIT=2", LIMIT),
        "{}",
        screen.plain_screen()
    );
    let _ = screen.finish(LIMIT);
    assert!(!sandbox.has_secret("OpenAI"));
}

#[test]
fn the_setup_key_prompt_shows_stars_too() {
    let sandbox = Sandbox::new();
    let mut screen = terminal(&sandbox, "cleanping setup");
    assert!(screen.wait_for_text("Step 1:", LIMIT));
    screen.send("1\n");
    assert!(
        screen.wait_for_text("API key (typing is hidden)", LIMIT),
        "{}",
        screen.plain_screen()
    );
    std::thread::sleep(SETTLE);
    screen.send("abcdef");
    assert!(
        screen.wait_for_text("******", LIMIT),
        "{}",
        screen.plain_screen()
    );
    screen.send("\x03");
    let _ = screen.finish(LIMIT);
}
