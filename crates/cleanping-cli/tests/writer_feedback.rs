//! What the writer says after Enter must still be on a real screen once the next prompt is drawn.
//! The prompt redraw moves the cursor up one line and clears everything below it, so a message
//! printed on the line just above the prompt is wiped at once. `plain_screen` strips those control
//! codes, so this test reads the raw output and checks where the cursor goes.
#![cfg(target_os = "linux")]

mod support;

use support::shell::available;
use support::writer::*;
use support::*;

const COPIED: &str = "Copied. Paste it anywhere (Cmd+V on a Mac, Ctrl+V elsewhere).";
const MOVE_UP: &str = "\x1b[A";

#[test]
fn the_message_after_enter_is_not_erased_by_the_next_prompt() {
    if !available("zsh") {
        return;
    }
    let sandbox = Sandbox::new();
    let stubs = sandbox.dir.path().join("stubs");
    std::fs::create_dir_all(&stubs).unwrap();
    stub_tool(
        &stubs,
        "pbcopy",
        &sandbox.dir.path().join("clipboard.txt"),
        0,
    );
    let path = format!("{}:{}", stubs.display(), sandbox.path_with_binary());
    let mut screen = start(&sandbox, &path, &[]);
    wait_for_prompt(&screen);
    screen.send("hello there");
    screen.send(ENTER);
    assert!(
        screen.wait_for_text(COPIED, LIMIT),
        "no message after Enter: {}",
        screen.plain_screen()
    );
    // Wait for the next prompt to be drawn after the message.
    let redrawn = || {
        let raw = screen.screen();
        raw.find(COPIED).is_some_and(|at| {
            let after = &raw[at..];
            after
                .find(MOVE_UP)
                .is_some_and(|up| after[up..].contains(PROMPT))
        })
    };
    assert!(
        screen.wait_until(LIMIT, redrawn),
        "the prompt never came back"
    );
    std::thread::sleep(SETTLE);
    let raw = screen.screen();
    let after = &raw[raw.find(COPIED).unwrap() + COPIED.len()..];
    let redraw_at = after.find(MOVE_UP).expect("the prompt redraw moves up");
    let lines_before_redraw = after[..redraw_at].matches('\n').count();
    assert!(
        lines_before_redraw >= 2,
        "the redraw moves up onto the message line and clears it; newlines before it: {lines_before_redraw}"
    );
}

/// The escape that turns on "standout" (reverse video), which the shell key uses to light up changed
/// words. In the writer the words stay lit on the lines that scrolled away, which looks broken.
const HIGHLIGHT_ON: &str = "\x1b[7m";

#[test]
fn the_fixed_text_is_not_highlighted() {
    if !available("zsh") {
        return;
    }
    let server = serve(vec![ok_reply("Please fix this.")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let stubs = sandbox.dir.path().join("stubs");
    std::fs::create_dir_all(&stubs).unwrap();
    let record = sandbox.dir.path().join("clipboard.txt");
    stub_tool(&stubs, "pbcopy", &record, 0);
    let path = format!("{}:{}", stubs.display(), sandbox.path_with_binary());
    let mut screen = start(&sandbox, &path, &[]);
    wait_for_prompt(&screen);
    screen.send("pleae fix this");
    // Only what appears after the fix counts: zsh itself prints a reverse-video `%` at start-up.
    let before_fix = screen.screen().len();
    screen.send(FIX);
    assert!(
        screen.wait_for_text("Please fix this.", LIMIT),
        "the fix never appeared: {}",
        screen.plain_screen()
    );
    screen.send(ENTER);
    wait_for_file(&screen, &record, "Please fix this.");
    std::thread::sleep(SETTLE);
    let raw_all = screen.screen();
    let raw = &raw_all[before_fix..];
    if let Some(at) = raw.find(HIGHLIGHT_ON) {
        let from = raw[..at].char_indices().rev().nth(60).map_or(0, |(i, _)| i);
        let to = raw[at..]
            .char_indices()
            .nth(60)
            .map_or(raw.len(), |(i, _)| at + i);
        panic!(
            "the fixed words were highlighted and stay lit on the lines above; around it: {:?}",
            &raw[from..to]
        );
    }
}
