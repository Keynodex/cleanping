//! The shell key: rewrites the command line, a second press restores it, and it can never
//! turn a one-line command into several lines.
//! Drives a real interactive shell inside a pseudo-terminal (Linux `script`). Nothing is typed
//! until the shell shows its prompt, so no test depends on how fast a machine starts a shell.
#![cfg(target_os = "linux")]

mod support;

use support::shell::*;
use support::*;

#[test]
fn init_prints_a_script_per_shell_and_rejects_unknown_ones() {
    let sandbox = Sandbox::new();
    let zsh = sandbox.run(&["init", "zsh"], None);
    assert_eq!(zsh.code, 0);
    assert!(zsh.stdout.contains("_cleanping_polish") && zsh.stdout.contains("bindkey"));
    let bash = sandbox.run(&["init", "bash"], None);
    assert_eq!(bash.code, 0);
    assert!(bash.stdout.contains("_cleanping_polish") && bash.stdout.contains("bind -x"));
    assert_eq!(sandbox.run(&["init", "powershell"], None).code, 2);
}

#[test]
fn init_does_not_touch_the_database_or_the_key_file() {
    let sandbox = Sandbox::new();
    sandbox.run(&["init", "zsh"], None);
    assert!(
        !sandbox.db_path().exists(),
        "init must be instant and side-effect free"
    );
    assert!(
        !sandbox.dir.path().join("config").exists(),
        "init must not create the config directory either"
    );
}

fn press_rewrites_then_restores(flavor: &Flavor) {
    if !available(flavor.name) {
        return;
    }
    let server = serve(vec![ok_reply("Please fix this.")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let typed = "pleae fix this";
    let recorded = session(&sandbox, flavor).type_and_exit(&[typed, PRESS, DUMP, PRESS, DUMP]);
    assert_eq!(
        recorded, "[Please fix this.]\n[pleae fix this]\n",
        "{}",
        flavor.name
    );
    assert_eq!(server.user_text(0), typed, "the line is the user text");
    assert_eq!(
        server.requests.lock().unwrap().len(),
        1,
        "restoring must not call the API again"
    );
}

#[test]
fn zsh_key_rewrites_the_line_and_a_second_press_restores_it() {
    press_rewrites_then_restores(&ZSH);
}

#[test]
fn bash_key_rewrites_the_line_and_a_second_press_restores_it() {
    press_rewrites_then_restores(&BASH);
}

fn failure_keeps_the_line(flavor: &Flavor) {
    if !available(flavor.name) {
        return;
    }
    let server = serve(vec![status_reply(401, "secret server detail")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let recorded = session(&sandbox, flavor).type_and_exit(&["pleae fix this", PRESS, DUMP]);
    assert_eq!(
        recorded, "[pleae fix this]\n",
        "{}: a failed rewrite must leave the line alone",
        flavor.name
    );
    assert_eq!(
        server.requests.lock().unwrap().len(),
        1,
        "{}: one press, one request",
        flavor.name
    );
}

#[test]
fn zsh_failure_keeps_the_original_line() {
    failure_keeps_the_line(&ZSH);
}

#[test]
fn bash_failure_keeps_the_original_line() {
    failure_keeps_the_line(&BASH);
}

/// A hostile reply: its first part is a command and its last part looks harmless. On a
/// terminal only the tail would be visible, yet Enter would run the command.
fn hostile_reply_is_refused(flavor: &Flavor, hostile: &str) {
    if !available(flavor.name) {
        return;
    }
    let server = serve(vec![ok_reply(hostile)]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let mut shell = session(&sandbox, flavor);
    shell.enter("cd");
    let recorded = shell.type_and_exit(&["pleae fix this", PRESS, DUMP, ENTER]);
    assert_eq!(
        recorded, "[pleae fix this]\n",
        "{}: the line must stay as typed",
        flavor.name
    );
    assert_eq!(server.requests.lock().unwrap().len(), 1);
    assert!(
        !sandbox.dir.path().join("PWNED").exists(),
        "{}: the hidden command ran",
        flavor.name
    );
}

fn many_lines() -> String {
    format!("touch PWNED #{}\nPlease fix this.", "\n".repeat(80))
}

fn one_padded_line() -> String {
    format!("touch PWNED;{}Please fix this.", " ".repeat(3000))
}

#[test]
fn zsh_refuses_a_reply_with_more_lines_than_the_text() {
    hostile_reply_is_refused(&ZSH, &many_lines());
}

#[test]
fn bash_refuses_a_reply_with_more_lines_than_the_text() {
    hostile_reply_is_refused(&BASH, &many_lines());
}

#[test]
fn zsh_refuses_one_line_padded_with_blanks_to_push_its_start_off_screen() {
    hostile_reply_is_refused(&ZSH, &one_padded_line());
}

#[test]
fn bash_refuses_one_line_padded_with_blanks_to_push_its_start_off_screen() {
    hostile_reply_is_refused(&BASH, &one_padded_line());
}

/// Text that already has two lines may come back with two lines.
fn a_multi_line_text_may_come_back_multi_line(flavor: &Flavor) {
    if !available(flavor.name) {
        return;
    }
    let server = serve(vec![ok_reply("Line one.\nLine two.")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let recorded = session(&sandbox, flavor).type_and_exit(&[
        "line one",
        QUOTED_NEWLINE,
        "line two",
        PRESS,
        DUMP,
    ]);
    assert_eq!(recorded, "[Line one.\nLine two.]\n", "{}", flavor.name);
    assert_eq!(server.user_text(0), "line one\nline two");
}

#[test]
fn zsh_allows_as_many_lines_as_the_text_had() {
    a_multi_line_text_may_come_back_multi_line(&ZSH);
}

#[test]
fn bash_allows_as_many_lines_as_the_text_had() {
    a_multi_line_text_may_come_back_multi_line(&BASH);
}
