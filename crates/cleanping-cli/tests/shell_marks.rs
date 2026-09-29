//! What the shell key shows after a rewrite: zsh highlights the changed words, bash prints a
//! "was:" line with the original, and both refuse to send a line that holds a secret.
#![cfg(target_os = "linux")]

mod support;

use support::shell::*;
use support::*;

const PRESS: &str = "\x18\x10"; // Ctrl-X Ctrl-P, the default key
const DUMP: &str = "\x18\x04"; // Ctrl-X Ctrl-D, bound by session() to record the line
const MARKS: &str = "\x18\x08"; // Ctrl-X Ctrl-H, bound by these tests to record the highlights
const EDIT: &str = "\x18\x05"; // Ctrl-X Ctrl-E, bound by these tests to change the line

/// Looks like a real API key to the secret check, and is not one. Written in two pieces so no
/// line in this file looks like a real key to a secret scanner.
const FAKE_KEY: &str = concat!("sk-", "Zx81QmVt3LpRw92NcYb47HdKe06Fa5Ug");

/// zsh helpers: one records `region_highlight`, one appends "x" to the line the way typing
/// does and then runs the same clean-up the redraw hook runs.
const ZSH_HELPERS: &str = concat!(
    "_h() { print -r -- \"<${(j:|:)region_highlight}>\" >> \"$DUMP\"; }; zle -N _h; bindkey '^X^H' _h; ",
    "_e() { BUFFER+=x; _cleanping_unmark; }; zle -N _e; bindkey '^X^E' _e"
);

fn zsh_session(sandbox: &Sandbox) -> Shell {
    zsh_session_with(sandbox, &[])
}

fn zsh_session_with(sandbox: &Sandbox, extra: &[(&str, &str)]) -> Shell {
    let mut shell = Shell::start_with(sandbox, ZSH.command, extra);
    shell.enter(ZSH.init);
    shell.enter(ZSH.record_line);
    shell.enter(ZSH_HELPERS);
    shell
}

fn with_reply(reply: &str) -> (Sandbox, FakeServer) {
    let server = serve(vec![ok_reply(reply)]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    (sandbox, server)
}

#[test]
fn zsh_highlights_the_changed_words() {
    if !available("zsh") {
        return;
    }
    let (sandbox, _server) = with_reply("Please fix this.");
    let recorded = zsh_session(&sandbox).type_and_exit(&["pleae fix this", PRESS, MARKS]);
    assert_eq!(recorded, "<0 6 standout|11 16 standout>\n");
}

#[test]
fn zsh_highlights_accented_letters_in_the_right_place() {
    if !available("zsh") {
        return;
    }
    let (sandbox, _server) = with_reply("Café bien fine.");
    let shell = zsh_session_with(&sandbox, &[("LC_ALL", "C.UTF-8")]);
    let recorded = shell.type_and_exit(&["cafe bien fine", PRESS, DUMP, MARKS]);
    // "Café" is 4 characters but 5 bytes: a byte count would put the second range one place off.
    assert_eq!(
        recorded,
        "[Café bien fine.]\n<0 4 standout|10 15 standout>\n"
    );
}

#[test]
fn zsh_skips_the_highlight_when_it_counts_bytes_not_characters() {
    if !available("zsh") {
        return;
    }
    // No UTF-8 locale: zsh counts bytes, so character positions would light the wrong text.
    let (sandbox, _server) = with_reply("Café bien fine.");
    let shell = zsh_session_with(&sandbox, &[("LC_ALL", "C")]);
    let recorded = shell.type_and_exit(&["cafe bien fine", PRESS, DUMP, MARKS]);
    assert_eq!(recorded, "[Café bien fine.]\n<>\n");
}

#[test]
fn zsh_second_press_restores_the_line_and_removes_the_highlight() {
    if !available("zsh") {
        return;
    }
    let (sandbox, _server) = with_reply("Please fix this.");
    let recorded = zsh_session(&sandbox).type_and_exit(&["pleae fix this", PRESS, PRESS, MARKS]);
    assert_eq!(recorded, "<>\n");
}

#[test]
fn zsh_highlight_goes_away_once_the_line_is_edited() {
    if !available("zsh") {
        return;
    }
    let (sandbox, _server) = with_reply("Please fix this.");
    let recorded =
        zsh_session(&sandbox).type_and_exit(&["pleae fix this", PRESS, MARKS, EDIT, MARKS]);
    assert_eq!(recorded, "<0 6 standout|11 16 standout>\n<>\n");
}

#[test]
fn zsh_leaves_other_highlights_alone() {
    if !available("zsh") {
        return;
    }
    let (sandbox, _server) = with_reply("Please fix this.");
    let mut shell = zsh_session(&sandbox);
    // Another plugin's entry, added by a widget the way such plugins do it.
    shell.enter("_p() { region_highlight+=('0 1 underline'); }; zle -N _p; bindkey '^X^Y' _p");
    let recorded = shell.type_and_exit(&["pleae fix this", "\x18\x19", PRESS, EDIT, MARKS]);
    assert_eq!(recorded, "<0 1 underline>\n");
}

#[test]
fn zsh_removes_the_highlight_on_every_redraw_through_a_hook() {
    if !available("zsh") {
        return;
    }
    let (sandbox, _server) = with_reply("Please fix this.");
    let mut shell = zsh_session(&sandbox);
    shell.enter("add-zle-hook-widget -L >> \"$DUMP\"");
    let recorded = shell.type_and_exit(&[]);
    assert!(
        recorded.contains("line-pre-redraw") && recorded.contains("_cleanping_unmark"),
        "{recorded}"
    );
}

#[test]
fn the_highlight_can_be_turned_off() {
    if !available("zsh") {
        return;
    }
    let (sandbox, _server) = with_reply("Please fix this.");
    let mut shell = Shell::start_with(&sandbox, ZSH.command, &[("CLEANPING_HIGHLIGHT", "")]);
    shell.enter(ZSH.init);
    shell.enter(ZSH.record_line);
    shell.enter(ZSH_HELPERS);
    let recorded = shell.type_and_exit(&["pleae fix this", PRESS, DUMP, MARKS]);
    assert_eq!(recorded, "[Please fix this.]\n<>\n");
}

#[test]
fn bash_shows_what_the_line_was() {
    if !available("bash") {
        return;
    }
    let (sandbox, _server) = with_reply("Please fix this.");
    let mut shell = session(&sandbox, &BASH);
    shell.wait_for_prompt();
    shell.terminal.send(&format!("pleae fix this{PRESS}"));
    assert!(
        shell
            .terminal
            .wait_for_text("cleanping: was: pleae fix this", LIMIT),
        "{}",
        shell.terminal.plain_screen()
    );
}

/// A command line that holds a secret is never sent, and stays exactly as typed.
fn a_line_with_a_secret_is_not_sent(flavor: &Flavor) {
    if !available(flavor.name) {
        return;
    }
    let (sandbox, server) = with_reply("Cleaned.");
    let line = format!("export KEY={FAKE_KEY}");
    let recorded = session(&sandbox, flavor).type_and_exit(&[&line, PRESS, DUMP]);
    assert_eq!(recorded, format!("[{line}]\n"), "{}", flavor.name);
    assert_eq!(
        server.requests.lock().unwrap().len(),
        0,
        "{}: nothing may be sent",
        flavor.name
    );
}

#[test]
fn zsh_does_not_send_a_line_that_holds_a_secret() {
    a_line_with_a_secret_is_not_sent(&ZSH);
}

#[test]
fn bash_does_not_send_a_line_that_holds_a_secret() {
    a_line_with_a_secret_is_not_sent(&BASH);
}
