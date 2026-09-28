//! The shell key: rewrites the command line, a second press restores it, and it can never
//! turn a one-line command into several lines.
//! Drives a real interactive shell inside a pseudo-terminal (Linux `script`). Nothing is typed
//! until the shell shows its prompt, so no test depends on how fast a machine starts a shell.
#![cfg(target_os = "linux")]

mod support;

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use support::pty::Pty;
use support::*;

const PRESS: &str = "\x18\x10"; // Ctrl-X Ctrl-P, the default key
const DUMP: &str = "\x18\x04"; // Ctrl-X Ctrl-D, bound by the test to record the line
const KILL_LINE: &str = "\x15"; // Ctrl-U
const QUOTED_NEWLINE: &str = "\x16\n"; // Ctrl-V Ctrl-J: a real newline inside the line
const ENTER: &str = "\n";

/// The prompt every test shell uses, so the test can see when the shell is ready for input.
const READY: &str = "@@READY@@";
/// After a prompt appears the line editor still needs a moment to take over the terminal.
const SETTLE: Duration = Duration::from_millis(100);
const LIMIT: Duration = Duration::from_secs(20);

fn have(tool: &str) -> bool {
    Command::new("sh")
        .args(["-c", &format!("command -v {tool}")])
        .stdout(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Run a shell's tests only where it exists. On CI it must exist, or the tests prove nothing.
fn available(shell: &str) -> bool {
    if have(shell) {
        return true;
    }
    assert!(
        std::env::var_os("CI").is_none(),
        "{shell} must be installed on CI, or its tests would pass without running"
    );
    eprintln!("skipping: {shell} is not installed");
    false
}

/// One interactive shell in a pseudo-terminal.
struct Shell {
    terminal: Pty,
    dump: PathBuf,
    lines_entered: usize,
}

impl Shell {
    fn start(sandbox: &Sandbox, command: &str) -> Self {
        let dump = sandbox.dir.path().join("dump.txt");
        let env = sandbox.environment(
            Some(&sandbox.path_with_binary()),
            &[
                ("DUMP", dump.to_str().unwrap()),
                ("TERM", "xterm"),
                ("PS1", READY),
            ],
        );
        Self {
            terminal: Pty::start(&env, command),
            dump,
            lines_entered: 0,
        }
    }

    fn wait_for_prompt(&self) {
        let wanted = self.lines_entered + 1;
        let shown = || self.terminal.screen().matches(READY).count() >= wanted;
        assert!(
            self.terminal.wait_until(LIMIT, shown),
            "no prompt number {wanted}: {}",
            self.terminal.screen()
        );
        std::thread::sleep(SETTLE);
    }

    /// Type one complete line at a fresh prompt (setup such as `eval ...`).
    fn enter(&mut self, line: &str) {
        self.wait_for_prompt();
        self.terminal.send(&format!("{line}\n"));
        self.lines_entered += 1;
    }

    /// At a fresh prompt, type all `keys` in one go, discard the line, exit, and return what
    /// the test helper recorded. The keys are already queued when the key runs, so the
    /// shell reads them in order, however long the rewrite takes.
    fn type_and_exit(mut self, keys: &[&str]) -> String {
        self.wait_for_prompt();
        let mut typed = keys.concat();
        typed.push_str(KILL_LINE);
        typed.push_str("exit\n");
        self.terminal.send(&typed);
        self.terminal.finish(LIMIT);
        std::fs::read_to_string(&self.dump).unwrap_or_default()
    }
}

/// A shell flavor: how to start it and its two setup lines.
struct Flavor {
    name: &'static str,
    command: &'static str,
    init: &'static str,
    record_line: &'static str,
}

const ZSH: Flavor = Flavor {
    name: "zsh",
    command: "zsh -f -i",
    init: "eval \"$(cleanping init zsh)\"",
    record_line: "_d() { print -r -- \"[$BUFFER]\" >> \"$DUMP\"; }; zle -N _d; bindkey '^X^D' _d",
};

const BASH: Flavor = Flavor {
    name: "bash",
    command: "bash --noprofile --norc -i",
    init: "eval \"$(cleanping init bash)\"",
    record_line:
        "_d() { printf '[%s]\\n' \"$READLINE_LINE\" >> \"$DUMP\"; }; bind -x '\"\\C-x\\C-d\": _d'",
};

/// Start the flavor's shell with the cleanping key installed and the line recorder bound.
fn session(sandbox: &Sandbox, flavor: &Flavor) -> Shell {
    let mut shell = Shell::start(sandbox, flavor.command);
    shell.enter(flavor.init);
    shell.enter(flavor.record_line);
    shell
}

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
