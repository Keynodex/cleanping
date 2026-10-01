//! The progress line on stderr while a slow rewrite is on its way: shown only in a terminal,
//! only after a short delay, and always erased before the result or an error is printed.
//! Drives the real binary in a pseudo-terminal (Linux `script`) against a slow fake server.
#![cfg(target_os = "linux")]

mod support;

use std::time::{Duration, Instant};

use support::lines::shown_lines;
use support::pty::Pty;
use support::*;

const LIMIT: Duration = Duration::from_secs(20);
const ROUGH: &str = "pls fix teh login";
const CLEAN: &str = "Please fix the login.";
/// The delay before the line appears (`progress::bar::SHOW_AFTER` in the binary).
const SHOW_AFTER: Duration = Duration::from_millis(500);
/// Three times [`SHOW_AFTER`], so a slow reply would show the line if nothing hid it.
const SLOW: Duration = Duration::from_millis(1500);

fn terminal(sandbox: &Sandbox, command: &str, extra: &[(&str, &str)]) -> Pty {
    let mut variables = vec![("TERM", "xterm"), ("LANG", "C.UTF-8")];
    variables.extend_from_slice(extra);
    let env = sandbox.environment(Some(&sandbox.path_with_binary()), &variables);
    Pty::start(&env, &format!("stty rows 24 cols 80; {command}"))
}

fn with_server(server: &FakeServer) -> Sandbox {
    let sandbox = Sandbox::new();
    sandbox.add_local("local", server);
    sandbox
}

/// "about 40% 12s": a percent and whole seconds.
fn has_estimate(line: &str) -> bool {
    line.split("about ").skip(1).any(|rest| {
        let Some((percent, seconds)) = rest.split_once("% ") else {
            return false;
        };
        let seconds = seconds.split('s').next().unwrap_or("");
        !percent.is_empty()
            && percent.chars().all(|c| c.is_ascii_digit())
            && !seconds.is_empty()
            && seconds.chars().all(|c| c.is_ascii_digit())
    })
}

/// Run `command` against a held reply: wait for the line, then let the reply through. Returns
/// the visible lines once the command has ended and `last` (its final words) has been shown.
fn slow_run(reply: String, command: &str, last: &str) -> Vec<String> {
    let (server, release) = serve_held(reply);
    let sandbox = with_server(&server);
    let started = Instant::now();
    let mut screen = terminal(&sandbox, command, &[]);
    assert!(
        screen.wait_until(LIMIT, || has_estimate(&screen.plain_screen())),
        "no estimate: {:?}",
        screen.plain_screen()
    );
    assert!(started.elapsed() >= SHOW_AFTER, "too soon");
    let drawn = screen.plain_screen();
    assert!(drawn.contains("Fixing your text\u{2026} "), "{drawn:?}");
    assert!(drawn.contains('\u{25b1}'), "a bar: {drawn:?}");
    release.send(()).unwrap();
    assert!(screen.wait_for_exit(LIMIT).is_some(), "still running");
    assert!(
        screen.wait_for_text(last, LIMIT),
        "{:?}",
        screen.plain_screen()
    );
    shown_lines(&screen.plain_screen())
}

fn assert_no_bar(lines: &[String]) {
    assert!(
        lines
            .iter()
            .all(|line| !line.contains("about ") && !line.contains("Fixing")),
        "{lines:?}"
    );
}

#[test]
fn a_slow_rewrite_shows_the_estimate_and_erases_it_before_the_result() {
    let lines = slow_run(ok_reply(CLEAN), &format!("cleanping '{ROUGH}'"), CLEAN);
    assert!(lines.contains(&CLEAN.to_string()), "{lines:?}");
    assert_no_bar(&lines);
}

#[test]
fn the_line_is_erased_before_an_error_too() {
    let lines = slow_run(
        status_reply(500, "busy"),
        &format!("cleanping '{ROUGH}'"),
        "500",
    );
    assert!(
        lines.iter().any(|line| line.starts_with("cleanping: ")),
        "{lines:?}"
    );
    assert_no_bar(&lines);
}

#[test]
fn the_line_is_erased_before_a_refusal() {
    let command = format!("printf '%s' '{ROUGH}' | cleanping --keep-shape");
    let lines = slow_run(ok_reply("one\ntwo\nthree"), &command, "cleanping: ");
    assert!(
        lines.iter().any(|line| line.starts_with("cleanping: ")),
        "{lines:?}"
    );
    assert!(!lines.iter().any(|line| line == "one"), "{lines:?}");
    assert_no_bar(&lines);
}

/// The fake server answers at once on 127.0.0.1, a few milliseconds, far under [`SHOW_AFTER`].
/// The whole output is checked, not only what is left on screen: an erased line would leave
/// nothing on screen either, so only this proves it was never drawn.
#[test]
fn a_quick_reply_shows_nothing() {
    let server = serve(vec![ok_reply(CLEAN)]);
    let sandbox = with_server(&server);
    let mut screen = terminal(&sandbox, &format!("cleanping '{ROUGH}'"), &[]);
    assert_eq!(screen.wait_for_exit(LIMIT), Some(0));
    assert!(screen.wait_for_text(CLEAN, LIMIT));
    let raw = screen.plain_screen();
    assert!(
        !raw.contains("about ") && !raw.contains("Fixing"),
        "{raw:?}"
    );
    let lines = shown_lines(&raw);
    assert!(lines.contains(&CLEAN.to_string()), "{lines:?}");
}

#[test]
fn the_line_never_reaches_stdout() {
    let (server, release) = serve_held(ok_reply(CLEAN));
    let sandbox = with_server(&server);
    let command = format!("cleanping '{ROUGH}' > \"$HOME/out.txt\"");
    let mut screen = terminal(&sandbox, &command, &[]);
    assert!(
        screen.wait_until(LIMIT, || has_estimate(&screen.plain_screen())),
        "positive control, the line is on the terminal: {:?}",
        screen.plain_screen()
    );
    release.send(()).unwrap();
    assert_eq!(screen.wait_for_exit(LIMIT), Some(0));
    let stdout = std::fs::read_to_string(sandbox.dir.path().join("out.txt")).unwrap();
    assert_eq!(stdout, format!("{CLEAN}\n"));
    assert_no_bar(&shown_lines(&screen.plain_screen()));
}

#[test]
fn nothing_is_shown_when_stderr_is_not_a_terminal() {
    let server = serve_slow(ok_reply(CLEAN), SLOW);
    let sandbox = with_server(&server);
    let out = sandbox.run_with_env(&[ROUGH], None, &[("TERM", "xterm"), ("LANG", "C.UTF-8")]);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert_eq!(out.stdout, format!("{CLEAN}\n"));
    assert_eq!(out.stderr, "");
}

#[test]
fn nothing_is_shown_in_a_terminal_when_it_is_turned_off() {
    let server = serve_slow(ok_reply(CLEAN), SLOW);
    let sandbox = with_server(&server);
    let command = format!("cleanping '{ROUGH}'");
    let mut screen = terminal(&sandbox, &command, &[("CLEANPING_PROGRESS", "off")]);
    assert_eq!(screen.wait_for_exit(LIMIT), Some(0));
    assert!(screen.wait_for_text(CLEAN, LIMIT));
    let raw = screen.plain_screen();
    assert!(raw.contains(CLEAN), "{raw:?}");
    assert!(!raw.contains("about "), "{raw:?}");
}

#[test]
fn the_edit_screen_shows_the_estimate_while_it_waits() {
    let (server, release) = serve_held(ok_reply(CLEAN));
    let sandbox = with_server(&server);
    let file = sandbox.dir.path().join("prompt.txt");
    std::fs::write(&file, ROUGH).unwrap();
    let command = format!("cleanping edit '{}'", file.display());
    let mut screen = terminal(&sandbox, &command, &[]);
    assert!(
        screen.wait_until(LIMIT, || has_estimate(&screen.plain_screen())),
        "no estimate: {:?}",
        screen.plain_screen()
    );
    release.send(()).unwrap();
    assert!(screen.wait_for_text("Text edit complete", LIMIT));
    screen.send("\r");
    assert_eq!(screen.wait_for_exit(LIMIT), Some(0));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), CLEAN);
}

#[test]
fn edit_yes_in_a_terminal_shows_it_and_erases_it_too() {
    let command = format!(
        "printf '%s' '{ROUGH}' > \"$HOME/p.txt\"; cleanping edit --yes \"$HOME/p.txt\"; cat \"$HOME/p.txt\""
    );
    let lines = slow_run(ok_reply(CLEAN), &command, CLEAN);
    assert!(lines.contains(&CLEAN.to_string()), "{lines:?}");
    assert_no_bar(&lines);
}
