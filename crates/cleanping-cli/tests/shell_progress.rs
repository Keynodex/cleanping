//! The shell key and `cleanping writer` never show the progress line: they send cleanping's
//! stderr to a private file, which is not a terminal, so nothing can get in the way of the line
//! editor's redraw. `tests/progress.rs` is the positive control: the same slow reply in a
//! terminal does show it. Linux only (`script`).
#![cfg(target_os = "linux")]

mod support;

use std::time::Duration;

use support::shell::*;
use support::writer;
use support::*;

/// Three times the delay before the line would appear (0.5 s, `progress::bar::SHOW_AFTER`).
const SLOW: Duration = Duration::from_millis(1500);
const TYPED: &str = "pleae fix this";
const FIXED: &str = "Please fix this.";
/// A UTF-8 terminal, where the line would be drawn if it were shown.
const LOCALE: (&str, &str) = ("LANG", "C.UTF-8");

fn no_line(screen: &str) {
    assert!(
        !screen.contains("about ") && !screen.contains("Fixing your text"),
        "{screen:?}"
    );
}

fn slow_shell_key(flavor: &Flavor) {
    if !available(flavor.name) {
        return;
    }
    let server = serve_slow(ok_reply(FIXED), SLOW);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let mut shell = Shell::start_with(&sandbox, flavor.command, &[LOCALE]);
    shell.enter(flavor.init);
    shell.enter(flavor.record_line);
    shell.wait_for_prompt();
    shell
        .terminal
        .send(&format!("{TYPED}{PRESS}{DUMP}{KILL_LINE}exit\n"));
    assert!(
        shell.terminal.wait_for_exit(LIMIT).is_some(),
        "{}",
        flavor.name
    );
    let recorded = std::fs::read_to_string(&shell.dump).unwrap_or_default();
    assert_eq!(recorded, format!("[{FIXED}]\n"), "{}", flavor.name);
    no_line(&shell.terminal.plain_screen());
}

#[test]
fn the_zsh_key_shows_no_progress_line_even_for_a_slow_reply() {
    slow_shell_key(&ZSH);
}

#[test]
fn the_bash_key_shows_no_progress_line_even_for_a_slow_reply() {
    slow_shell_key(&BASH);
}

#[test]
fn the_writer_shows_no_progress_line_even_for_a_slow_reply() {
    if !available("zsh") {
        return;
    }
    let server = serve_slow(ok_reply(FIXED), SLOW);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let mut screen = writer::start(&sandbox, &sandbox.path_with_binary(), &[LOCALE]);
    writer::wait_for_prompt(&screen);
    screen.send(TYPED);
    screen.send(writer::FIX);
    assert!(
        screen.wait_for_text(FIXED, LIMIT),
        "{}",
        screen.plain_screen()
    );
    no_line(&screen.plain_screen());
}
