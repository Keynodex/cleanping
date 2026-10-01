//! `cleanping writer` fixes with the shell key (Ctrl+G), so a reply that changes a command on the
//! line is refused there too: the typed text stays and Enter copies it unchanged. Prose whose
//! apostrophes the fix adds is still fixed. Fake clipboard tool and fake API server only.
#![cfg(target_os = "linux")]

mod support;

use support::shell::available;
use support::writer::*;
use support::*;

/// Type `typed` in the writer, fix it with `reply` waiting, wait for `wait_for` if given, press
/// Enter, and check what reached the clipboard.
fn fix_then_copy(typed: &str, reply: &str, wait_for: Option<&str>, copied: &str) {
    if !available("zsh") {
        return;
    }
    let server = serve(vec![ok_reply(reply)]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let stubs = sandbox.dir.path().join("stubs");
    std::fs::create_dir_all(&stubs).unwrap();
    let record = sandbox.dir.path().join("clipboard.txt");
    stub_tool(&stubs, "pbcopy", &record, 0);
    let path = format!("{}:{}", stubs.display(), sandbox.path_with_binary());
    let mut screen = start(&sandbox, &path, &[]);
    wait_for_prompt(&screen);
    screen.send(typed);
    screen.send(FIX);
    if let Some(text) = wait_for {
        assert!(
            screen.wait_for_text(text, LIMIT),
            "no {text:?}: {}",
            screen.plain_screen()
        );
    }
    screen.send(ENTER);
    wait_for_file(&screen, &record, copied);
    assert_eq!(server.requests.lock().unwrap().len(), 1);
}

#[test]
fn a_reply_that_closes_an_open_quote_is_refused_and_the_text_is_kept() {
    let typed = "curl -d '{\"name\": \"lamp\"} why is it waiting";
    let closed = "curl -d '{\"name\": \"lamp\"}' Why is it waiting?";
    fix_then_copy(typed, closed, Some("changed your command"), typed);
}

#[test]
fn prose_gets_its_apostrophes_and_is_fixed() {
    fix_then_copy("i dont know", "I don't know.", None, "I don't know.");
}
