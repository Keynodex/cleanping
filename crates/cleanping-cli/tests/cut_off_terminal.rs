//! A reply cut off at the provider's length limit never replaces the user's text: not on a
//! shell line (zsh and bash), not on the `edit` screen, and not with `edit --yes`.
//! Drives real programs inside a pseudo-terminal (Linux `script`), waiting for what they show.
#![cfg(target_os = "linux")]

mod support;

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use support::pty::Pty;
use support::shell::*;
use support::*;

const TYPED: &str = "pleae fix this long text";
const HALF: &str = "Please fix this lo";
const WAIT: Duration = Duration::from_secs(20);

fn with_cut_off_server() -> (Sandbox, FakeServer) {
    let server = serve(vec![cut_off_reply(HALF)]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    (sandbox, server)
}

fn the_shell_line_stays_as_typed(flavor: &Flavor) {
    if !available(flavor.name) {
        return;
    }
    let (sandbox, server) = with_cut_off_server();
    let mut shell = session(&sandbox, flavor);
    shell.wait_for_prompt();
    shell.terminal.send(&format!("{TYPED}{PRESS}"));
    assert!(
        shell.terminal.wait_for_text("cut off", WAIT),
        "{}: the cut-off reply was used, no reason shown: {}",
        flavor.name,
        shell.terminal.plain_screen()
    );
    let recorded = shell.type_and_exit(&[DUMP]);
    assert_eq!(recorded, format!("[{TYPED}]\n"), "{}", flavor.name);
    assert_eq!(server.requests.lock().unwrap().len(), 1, "{}", flavor.name);
}

#[test]
fn zsh_keeps_the_typed_line_when_the_reply_was_cut_off() {
    the_shell_line_stays_as_typed(&ZSH);
}

#[test]
fn bash_keeps_the_typed_line_when_the_reply_was_cut_off() {
    the_shell_line_stays_as_typed(&BASH);
}

fn file_with_text(sandbox: &Sandbox) -> PathBuf {
    let file = sandbox.dir.path().join("prompt.txt");
    std::fs::write(&file, format!("{TYPED}\n")).unwrap();
    file
}

#[test]
fn the_edit_screen_says_why_and_keeps_the_text() {
    let (sandbox, _server) = with_cut_off_server();
    let file = file_with_text(&sandbox);
    let env = sandbox.environment(Some(&sandbox.path_with_binary()), &[("TERM", "xterm")]);
    let command = format!("stty rows 24 cols 80; cleanping edit '{}'", file.display());
    let mut screen = Pty::start(&env, &command);
    assert!(
        screen.wait_for_text("Could not edit", WAIT),
        "the cut-off reply was used: {}",
        screen.plain_screen()
    );
    // The header comes first, in its own writes; the footer is the last line drawn, so once it
    // is there the message and the text above it are too.
    assert!(
        screen.wait_for_text("Press any key to keep your text as it is", WAIT),
        "{}",
        screen.plain_screen()
    );
    let shown = screen.plain_screen();
    assert!(shown.contains("The reply was cut off"), "{shown}");
    assert!(
        shown.contains(TYPED),
        "the text must stay on screen: {shown}"
    );
    assert!(!shown.contains(HALF), "the reply leaked: {shown}");
    screen.send("x");
    assert_eq!(screen.finish(WAIT), 0, "a failure on the screen exits 0");
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        format!("{TYPED}\n")
    );
}

#[test]
fn edit_yes_leaves_the_file_untouched() {
    let (sandbox, _server) = with_cut_off_server();
    let file = file_with_text(&sandbox);
    // No terminal at all, as when a host app runs it in the background.
    let output = Command::new("setsid")
        .arg("-w")
        .arg(env!("CARGO_BIN_EXE_cleanping"))
        .args(["edit", "--yes"])
        .arg(&file)
        .env_clear()
        .envs(sandbox.environment(None, &[]))
        .stdin(Stdio::null())
        .output()
        .expect("setsid is needed for terminal tests");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(1),
        "the cut-off reply was used: {stderr}"
    );
    assert!(stderr.contains("The reply was cut off"), "{stderr}");
    assert!(!stderr.contains(HALF), "the reply leaked: {stderr}");
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        format!("{TYPED}\n")
    );
}
