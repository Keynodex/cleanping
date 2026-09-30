//! The shell key refuses a reply that changes the command on the line: a closed quote, a dropped
//! flag or a changed path. The typed line stays, and a message says why.
//! Drives a real interactive shell inside a pseudo-terminal (Linux `script`).
#![cfg(target_os = "linux")]

mod support;

use support::shell::*;
use support::*;

const TYPED: &str = "curl -d '{\"name\": \"lamp\"}";
const REFUSED: &str = "changed your command";

/// Type `TYPED`, press the key with `reply` waiting, and return what the line holds afterwards,
/// once `wait_for` (if any) is on the screen.
fn press(flavor: &Flavor, reply: &str, wait_for: Option<&str>) -> Option<String> {
    if !available(flavor.name) {
        return None;
    }
    let server = serve(vec![ok_reply(reply)]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let mut shell = session(&sandbox, flavor);
    shell.wait_for_prompt();
    shell.terminal.send(&format!("{TYPED}{PRESS}"));
    if let Some(text) = wait_for {
        assert!(
            shell.terminal.wait_for_text(text, LIMIT),
            "{}: no {text:?} on the screen: {}",
            flavor.name,
            shell.terminal.plain_screen()
        );
    }
    let recorded = shell.type_and_exit(&[DUMP]);
    assert_eq!(server.requests.lock().unwrap().len(), 1, "{}", flavor.name);
    Some(recorded)
}

fn a_closed_quote_is_refused(flavor: &Flavor) {
    let closed = format!("{TYPED}'");
    if let Some(line) = press(flavor, &closed, Some(REFUSED)) {
        assert_eq!(
            line,
            format!("[{TYPED}]\n"),
            "{}: the line must stay",
            flavor.name
        );
    }
}

#[test]
fn zsh_refuses_a_reply_that_closes_the_open_quote() {
    a_closed_quote_is_refused(&ZSH);
}

#[test]
fn bash_refuses_a_reply_that_closes_the_open_quote() {
    a_closed_quote_is_refused(&BASH);
}

/// The positive control: the same setup applies a reply that keeps the quote open.
fn a_kept_quote_is_applied(flavor: &Flavor) {
    let kept = "curl -d '{\"name\": \"Lamp\"}";
    if let Some(line) = press(flavor, kept, None) {
        assert_eq!(line, format!("[{kept}]\n"), "{}", flavor.name);
    }
}

#[test]
fn zsh_applies_a_reply_that_keeps_the_command() {
    a_kept_quote_is_applied(&ZSH);
}

#[test]
fn bash_applies_a_reply_that_keeps_the_command() {
    a_kept_quote_is_applied(&BASH);
}
