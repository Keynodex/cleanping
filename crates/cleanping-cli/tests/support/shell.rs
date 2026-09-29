//! A real interactive shell (zsh or bash) inside a pseudo-terminal, for testing the shell key.
//! Nothing is typed until the shell shows its prompt, so no test depends on how fast a machine
//! starts a shell.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use super::pty::Pty;
use super::Sandbox;

pub const PRESS: &str = "\x18\x10"; // Ctrl-X Ctrl-P, the default key
pub const DUMP: &str = "\x18\x04"; // Ctrl-X Ctrl-D, bound by the test to record the line
pub const KILL_LINE: &str = "\x15"; // Ctrl-U
pub const QUOTED_NEWLINE: &str = "\x16\n"; // Ctrl-V Ctrl-J: a real newline inside the line
pub const ENTER: &str = "\n";

/// The prompt every test shell uses, so the test can see when the shell is ready for input.
pub const READY: &str = "@@READY@@";
/// After a prompt appears the line editor still needs a moment to take over the terminal.
pub const SETTLE: Duration = Duration::from_millis(100);
pub const LIMIT: Duration = Duration::from_secs(20);

fn have(tool: &str) -> bool {
    Command::new("sh")
        .args(["-c", &format!("command -v {tool}")])
        .stdout(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Run a shell's tests only where it exists. On CI it must exist, or the tests prove nothing.
pub fn available(shell: &str) -> bool {
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
pub struct Shell {
    pub terminal: Pty,
    pub dump: PathBuf,
    lines_entered: usize,
}

impl Shell {
    pub fn start(sandbox: &Sandbox, command: &str) -> Self {
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

    pub fn wait_for_prompt(&self) {
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
    pub fn enter(&mut self, line: &str) {
        self.wait_for_prompt();
        self.terminal.send(&format!("{line}\n"));
        self.lines_entered += 1;
    }

    /// At a fresh prompt, type all `keys` in one go, discard the line, exit, and return what
    /// the test helper recorded. The keys are already queued when the key runs, so the
    /// shell reads them in order, however long the rewrite takes.
    pub fn type_and_exit(mut self, keys: &[&str]) -> String {
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
pub struct Flavor {
    pub name: &'static str,
    pub command: &'static str,
    pub init: &'static str,
    pub record_line: &'static str,
}

pub const ZSH: Flavor = Flavor {
    name: "zsh",
    command: "zsh -f -i",
    init: "eval \"$(cleanping init zsh)\"",
    record_line: "_d() { print -r -- \"[$BUFFER]\" >> \"$DUMP\"; }; zle -N _d; bindkey '^X^D' _d",
};

pub const BASH: Flavor = Flavor {
    name: "bash",
    command: "bash --noprofile --norc -i",
    init: "eval \"$(cleanping init bash)\"",
    record_line:
        "_d() { printf '[%s]\\n' \"$READLINE_LINE\" >> \"$DUMP\"; }; bind -x '\"\\C-x\\C-d\": _d'",
};

/// Start the flavor's shell with the cleanping key installed and the line recorder bound.
pub fn session(sandbox: &Sandbox, flavor: &Flavor) -> Shell {
    let mut shell = Shell::start(sandbox, flavor.command);
    shell.enter(flavor.init);
    shell.enter(flavor.record_line);
    shell
}
