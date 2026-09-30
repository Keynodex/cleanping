//! A real `cleanping writer` session inside a pseudo-terminal (Linux `script`). Nothing is typed
//! until the writer shows its prompt, so no test depends on how fast a machine starts zsh.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use super::pty::Pty;
use super::Sandbox;

pub const FIX: &str = "\x07"; // Ctrl+G
pub const ENTER: &str = "\r";
pub const EOF: &str = "\x04"; // Ctrl+D
pub const PROMPT: &str = "> ";
pub const LIMIT: Duration = Duration::from_secs(20);
/// After the prompt appears the line editor still needs a moment to take over the terminal.
pub const SETTLE: Duration = Duration::from_millis(150);

/// The full path of a program on this machine, or `None` when it is not installed.
pub fn locate(program: &str) -> Option<PathBuf> {
    let out = Command::new("sh")
        .args(["-c", &format!("command -v {program}")])
        .output()
        .ok()?;
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (out.status.success() && !path.is_empty()).then(|| PathBuf::from(path))
}

/// A folder holding only links to `programs`, so a PATH made of it finds nothing else.
pub fn bare_dir(sandbox: &Sandbox, programs: &[&str]) -> PathBuf {
    let dir = sandbox.dir.path().join("bare");
    std::fs::create_dir_all(&dir).unwrap();
    for program in programs {
        let target = locate(program).unwrap_or_else(|| panic!("{program} is needed"));
        std::os::unix::fs::symlink(target, dir.join(program)).unwrap();
    }
    dir
}

/// Put a fake clipboard tool `name` in `dir`: it saves its input to `record` and exits `status`.
pub fn stub_tool(dir: &Path, name: &str, record: &Path, status: i32) {
    use std::os::unix::fs::PermissionsExt;
    let script = format!("#!/bin/sh\ncat > '{}'\nexit {status}\n", record.display());
    let path = dir.join(name);
    std::fs::write(&path, script).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// `cleanping writer` in a terminal with the given PATH and extra environment variables.
pub fn start(sandbox: &Sandbox, path: &str, extra: &[(&str, &str)]) -> Pty {
    run_in_terminal(sandbox, path, extra, "cleanping writer")
}

/// Any shell command line in a terminal, with the same scrubbed environment.
pub fn run_in_terminal(
    sandbox: &Sandbox,
    path: &str,
    extra: &[(&str, &str)],
    command: &str,
) -> Pty {
    let mut variables = vec![("TERM", "xterm")];
    variables.extend_from_slice(extra);
    Pty::start(&sandbox.environment(Some(path), &variables), command)
}

/// Wait until the writer shows its prompt, then a moment longer.
pub fn wait_for_prompt(terminal: &Pty) {
    let shown = || terminal.plain_screen().ends_with(PROMPT);
    assert!(
        terminal.wait_until(LIMIT, shown),
        "the writer never showed its prompt: {}",
        terminal.plain_screen()
    );
    std::thread::sleep(SETTLE);
}

/// Wait until `file` has exactly `wanted` in it.
pub fn wait_for_file(terminal: &Pty, file: &Path, wanted: &str) {
    let done = || std::fs::read_to_string(file).is_ok_and(|text| text == wanted);
    assert!(
        terminal.wait_until(LIMIT, done),
        "{} should hold {wanted:?} but holds {:?}; screen: {}",
        file.display(),
        std::fs::read_to_string(file).ok(),
        terminal.plain_screen()
    );
}
