//! The keys of `cleanping writer`: Enter copies and never runs, Ctrl+G fixes in place. A fake
//! clipboard tool records what it is given; the fake API server answers Ctrl+G. No test here
//! can reach a real provider or a real clipboard.
#![cfg(target_os = "linux")]

mod support;

use std::path::PathBuf;

use support::pty::Pty;
use support::shell::available;
use support::writer::*;
use support::*;

const COPIED: &str = "Copied. Paste it anywhere (Cmd+V on a Mac, Ctrl+V elsewhere).";
const NOT_COPIED: &str =
    "Could not copy (no clipboard tool found). Select the text above and copy it yourself.";

/// A sandbox whose PATH starts with a fake `pbcopy` that exits `status` after saving its input.
struct Rig {
    sandbox: Sandbox,
    record: PathBuf,
    path: String,
}

fn rig(status: i32) -> Rig {
    let sandbox = Sandbox::new();
    let stubs = sandbox.dir.path().join("stubs");
    std::fs::create_dir_all(&stubs).unwrap();
    let record = sandbox.dir.path().join("clipboard.txt");
    stub_tool(&stubs, "pbcopy", &record, status);
    let path = format!("{}:{}", stubs.display(), sandbox.path_with_binary());
    Rig {
        sandbox,
        record,
        path,
    }
}

fn open(rig: &Rig) -> Pty {
    let screen = start(&rig.sandbox, &rig.path, &[]);
    wait_for_prompt(&screen);
    screen
}

#[test]
fn a_typed_command_is_never_run() {
    if !available("zsh") {
        return;
    }
    let rig = rig(0);
    let canary = rig.sandbox.dir.path().join("canary.txt");
    let typed = format!("echo CANARY > {}", canary.display());
    let mut screen = open(&rig);
    screen.send(&typed);
    screen.send(ENTER);
    wait_for_file(&screen, &rig.record, &typed);
    assert!(!canary.exists(), "a typed command was run");
}

#[test]
fn enter_copies_the_text_exactly_and_keeps_it_on_screen() {
    if !available("zsh") {
        return;
    }
    let rig = rig(0);
    let mut screen = open(&rig);
    screen.send("  say hello  ");
    screen.send(ENTER);
    wait_for_file(&screen, &rig.record, "  say hello  ");
    assert!(
        screen.wait_for_text(COPIED, LIMIT),
        "{}",
        screen.plain_screen()
    );
    let shown = screen.plain_screen();
    assert!(shown.find("say hello").unwrap() < shown.find(COPIED).unwrap());
    assert!(screen.wait_until(LIMIT, || screen.plain_screen().ends_with(PROMPT)));
}

#[test]
fn a_pasted_paragraph_is_copied_whole() {
    if !available("zsh") {
        return;
    }
    let rig = rig(0);
    let mut screen = open(&rig);
    screen.send("\x1b[200~line one\nline two\x1b[201~");
    screen.send(ENTER);
    wait_for_file(&screen, &rig.record, "line one\nline two");
}

#[test]
fn a_blank_line_copies_nothing() {
    if !available("zsh") {
        return;
    }
    let rig = rig(0);
    let mut screen = open(&rig);
    screen.send("   ");
    screen.send(ENTER);
    screen.send("\x15x"); // Ctrl+U clears the blanks, which Enter left alone

    screen.send(ENTER);
    wait_for_file(&screen, &rig.record, "x");
    // The message is printed after the copy, then the prompt comes back: wait for the prompt.
    assert!(
        screen.wait_until(LIMIT, || screen.plain_screen().ends_with(PROMPT)),
        "{}",
        screen.plain_screen()
    );
    assert_eq!(screen.plain_screen().matches("Copied.").count(), 1);
}

#[test]
fn a_failing_clipboard_tool_is_reported_and_the_line_is_cleared() {
    if !available("zsh") {
        return;
    }
    let rig = rig(1);
    let mut screen = open(&rig);
    screen.send("say hello");
    screen.send(ENTER);
    assert!(
        screen.wait_for_text(NOT_COPIED, LIMIT),
        "a failed copy must be reported: {}",
        screen.plain_screen()
    );
    assert!(!screen.plain_screen().contains("Copied."));
    // The next line is copied on its own, so the failed one is gone from the buffer.
    stub_tool(
        rig.path.split(':').next().unwrap().as_ref(),
        "pbcopy",
        &rig.record,
        0,
    );
    screen.send("next");
    screen.send(ENTER);
    wait_for_file(&screen, &rig.record, "next");
}

#[test]
fn with_no_clipboard_tool_it_says_so_and_carries_on() {
    if !available("zsh") {
        return;
    }
    let sandbox = Sandbox::new();
    let bare = bare_dir(&sandbox, &["script", "zsh", "cat"]);
    let binary = sandbox.path_with_binary();
    let path = format!("{}:{}", binary.split(':').next().unwrap(), bare.display());
    let mut screen = start(&sandbox, &path, &[]);
    wait_for_prompt(&screen);
    screen.send("say hello");
    screen.send(ENTER);
    assert!(
        screen.wait_for_text(NOT_COPIED, LIMIT),
        "{}",
        screen.plain_screen()
    );
    // Still running, buffer cleared, and a tool that appears later is found.
    let record = sandbox.dir.path().join("clipboard.txt");
    stub_tool(&bare, "pbcopy", &record, 0);
    screen.send("next");
    screen.send(ENTER);
    wait_for_file(&screen, &record, "next");
}

#[test]
fn ctrl_g_fixes_the_line_and_enter_copies_the_fixed_text() {
    if !available("zsh") {
        return;
    }
    let server = serve(vec![ok_reply("Please fix this.")]);
    let rig = rig(0);
    rig.sandbox.add_local("Local", &server);
    let mut screen = open(&rig);
    screen.send("pleae fix this");
    screen.send(FIX);
    screen.send(ENTER);
    wait_for_file(&screen, &rig.record, "Please fix this.");
    assert_eq!(server.user_text(0), "pleae fix this");
    assert_eq!(server.requests.lock().unwrap().len(), 1);
}

#[test]
fn exclamation_marks_are_plain_text() {
    if !available("zsh") {
        return;
    }
    let rig = rig(0);
    let mut screen = open(&rig);
    screen.send("echo !! hi !word");
    screen.send(ENTER);
    wait_for_file(&screen, &rig.record, "echo !! hi !word");
}

/// Every key that could accept a line copies it instead, in any keymap `$EDITOR` might pick.
#[test]
fn no_way_of_accepting_a_line_runs_it() {
    if !available("zsh") {
        return;
    }
    for editor in ["", "vi"] {
        let rig = rig(0);
        let canary = rig.sandbox.dir.path().join("canary.txt");
        let variables = [("EDITOR", editor), ("VISUAL", editor)];
        let mut screen = start(&rig.sandbox, &rig.path, &variables);
        wait_for_prompt(&screen);
        for (name, key) in [("enter", "\r"), ("ctrl-j", "\n"), ("ctrl-o", "\x0f")] {
            let typed = format!("echo {name} {editor} > {}", canary.display());
            screen.send(&typed);
            screen.send(key);
            wait_for_file(&screen, &rig.record, &typed);
            assert!(!canary.exists(), "{name} ran a command (EDITOR={editor:?})");
            assert!(screen.wait_until(LIMIT, || screen.plain_screen().ends_with(PROMPT)));
        }
    }
}

/// Suspend, the external editor and history search could leave the writer or run a line; these
/// keys do nothing.
#[test]
fn keys_that_could_leave_or_search_do_nothing() {
    if !available("zsh") {
        return;
    }
    let rig = rig(0);
    let editor = rig.sandbox.dir.path().join("editor-ran.txt");
    let stubs = std::path::Path::new(rig.path.split(':').next().unwrap()).to_path_buf();
    stub_tool(&stubs, "fake-editor", &editor, 0);
    let fake = stubs.join("fake-editor");
    let mut screen = start(
        &rig.sandbox,
        &rig.path,
        &[
            ("EDITOR", fake.to_str().unwrap()),
            ("VISUAL", fake.to_str().unwrap()),
        ],
    );
    wait_for_prompt(&screen);
    screen.send("keep me\x1a\x18\x05\x12\x13"); // ^Z ^X^E ^R ^S
    screen.send(ENTER);
    wait_for_file(&screen, &rig.record, "keep me");
    assert!(!editor.exists(), "the external editor was started");
}
