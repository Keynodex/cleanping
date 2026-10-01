//! `cleanping edit` when the edit changes a command in the text: the screen says so on the line
//! under the header, and `--yes` says so on stderr. Either way the edit can still be accepted.
//! Drives the real binary in a pseudo-terminal (Linux `script`), waiting for text, not time.
#![cfg(target_os = "linux")]

mod support;

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use support::pty::Pty;
use support::*;

const LIMIT: Duration = Duration::from_secs(20);
const ASKED: &str = "why is it waiting?\n```\ncurl -d '{\"a\": 1}\n```\n";
const CLOSED: &str = "Why is it waiting?\n```\ncurl -d '{\"a\": 1}'\n```";
const NOTICE: &str = "Check the command: a quote was closed in the command starting";

fn setup(reply: &str) -> (Sandbox, PathBuf) {
    let sandbox = Sandbox::new();
    sandbox.add_local("local", &serve(vec![ok_reply(reply)]));
    let file = sandbox.dir.path().join("prompt.txt");
    std::fs::write(&file, ASKED).unwrap();
    (sandbox, file)
}

fn screen(sandbox: &Sandbox, file: &std::path::Path) -> Pty {
    let env = sandbox.environment(Some(&sandbox.path_with_binary()), &[("TERM", "xterm")]);
    let command = format!("stty rows 24 cols 120; cleanping edit '{}'", file.display());
    Pty::start(&env, &command)
}

#[test]
fn the_screen_names_a_changed_command_and_enter_still_accepts() {
    let (sandbox, file) = setup(CLOSED);
    let mut screen = screen(&sandbox, &file);
    assert!(
        screen.wait_for_text("Text edit complete", LIMIT),
        "{}",
        screen.plain_screen()
    );
    assert!(
        screen.wait_for_text(NOTICE, LIMIT),
        "{}",
        screen.plain_screen()
    );
    screen.send("\r");
    assert_eq!(screen.finish(LIMIT), 0);
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        format!("{CLOSED}\n")
    );
}

#[test]
fn an_unchanged_command_shows_no_warning() {
    let reply = "Why is it waiting?\n```\ncurl -d '{\"a\": 1}\n```";
    let (sandbox, file) = setup(reply);
    let mut screen = screen(&sandbox, &file);
    assert!(screen.wait_for_text("Text edit complete", LIMIT));
    // The warning line sits above the footer, which is drawn last: wait for the footer so the
    // check below sees the whole screen.
    assert!(
        screen.wait_for_text("Enter accept", LIMIT),
        "{}",
        screen.plain_screen()
    );
    assert!(!screen.plain_screen().contains("Check the command"));
    screen.send("n");
    assert_eq!(screen.finish(LIMIT), 0);
}

#[test]
fn yes_writes_the_edit_and_warns_on_stderr() {
    let (sandbox, file) = setup(CLOSED);
    let output = Command::new("setsid")
        .arg("-w")
        .arg(env!("CARGO_BIN_EXE_cleanping"))
        .args(["edit", "--yes"])
        .arg(&file)
        .env_clear()
        .envs(sandbox.environment(None, &[]))
        .stdin(std::process::Stdio::null())
        .output()
        .expect("setsid is needed for terminal tests");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    assert!(
        stderr.contains("cleanping: warning: a quote was closed in the command starting"),
        "{stderr}"
    );
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        format!("{CLOSED}\n")
    );
}
