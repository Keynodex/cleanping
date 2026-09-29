//! `cleanping setup`, the real binary in a real terminal (Linux `script`): every step waits for
//! the question it answers, never for a fixed time. No test here can reach a real provider.
#![cfg(target_os = "linux")]

mod support;

use std::path::Path;
use std::time::Duration;

use support::pty::Pty;
use support::*;

const LIMIT: Duration = Duration::from_secs(20);
/// After a hidden prompt appears the terminal needs a moment to stop echoing.
const SETTLE: Duration = Duration::from_millis(150);
/// Not a real key.
const SECRET: &str = "e2e-not-a-real-key-4242";

fn terminal(sandbox: &Sandbox, path: Option<&str>) -> Pty {
    let path = path.map_or_else(|| sandbox.path_with_binary(), str::to_string);
    let env = sandbox.environment(Some(&path), &[("TERM", "xterm"), ("SHELL", "/usr/bin/zsh")]);
    Pty::start(&env, "cleanping setup")
}

/// Wait for the question, then answer it.
fn answer(screen: &mut Pty, question: &str, reply: &str) {
    assert!(
        screen.wait_for_text(question, LIMIT),
        "waiting for {question:?}: {}",
        screen.plain_screen()
    );
    screen.send(reply);
}

#[test]
fn setup_needs_a_terminal_and_says_what_to_do_instead() {
    let out = Sandbox::new().run(&["setup"], None);
    assert_eq!(
        out.code, 2,
        "must refuse without a terminal: {}",
        out.stderr
    );
    assert!(out.stderr.contains("needs a terminal"), "{}", out.stderr);
    assert!(out.stderr.contains("keys add"), "{}", out.stderr);
}

#[test]
fn a_first_run_saves_the_provider_tests_it_and_shows_how_to_use_it() {
    // The first connection is the check "is this Ollama?"; the second is the test itself.
    let server = serve(vec![status_reply(404, "no"), ok_reply("OK")]);
    let sandbox = Sandbox::new();
    let mut screen = terminal(&sandbox, None);
    answer(&mut screen, "Step 1: choose your AI", "5\n");
    answer(&mut screen, "Name for this key", "Local\n");
    answer(&mut screen, "API address", &format!("{}\n", server.url));
    answer(&mut screen, "Model name:", "test-model\n");
    answer(&mut screen, "Step 2:", "\n");
    answer(&mut screen, "Step 3:", "y\n");
    assert!(
        screen.wait_for_text("All set", LIMIT),
        "{}",
        screen.plain_screen()
    );
    assert!(screen.wait_for_text("cleanping setup again", LIMIT));
    assert_eq!(screen.finish(LIMIT), 0);
    let shown = sandbox.run(&["keys", "list"], None).stdout;
    assert!(
        shown.contains("Local") && shown.contains("test-model"),
        "{shown}"
    );
    assert_eq!(server.user_text(1), "ping");
}

#[test]
fn the_screen_shows_the_lines_to_add_and_changes_no_file() {
    let server = serve(vec![status_reply(404, "no"), ok_reply("OK")]);
    let sandbox = Sandbox::new();
    std::fs::write(sandbox.dir.path().join(".zshrc"), "# mine\n").unwrap();
    let mut screen = terminal(&sandbox, None);
    answer(&mut screen, "Step 1:", "5\n");
    answer(&mut screen, "Name for this key", "Local\n");
    answer(&mut screen, "API address", &format!("{}\n", server.url));
    answer(&mut screen, "Model name:", "m\n");
    answer(&mut screen, "Step 2:", "\n");
    answer(&mut screen, "Step 3:", "y\n");
    assert!(screen.wait_for_text("All set", LIMIT));
    let text = screen.plain_screen();
    assert!(text.contains("export VISUAL=\"cleanping edit\""), "{text}");
    assert!(
        text.contains("~/.zshrc") && text.contains("init zsh"),
        "{text}"
    );
    assert_eq!(screen.finish(LIMIT), 0);
    let profile = std::fs::read_to_string(sandbox.dir.path().join(".zshrc")).unwrap();
    assert_eq!(profile, "# mine\n", "setup must not edit the profile");
}

#[test]
fn a_typed_key_is_hidden_and_saved_privately() {
    let sandbox = Sandbox::new();
    let mut screen = terminal(&sandbox, None);
    answer(&mut screen, "Step 1:", "1\n"); // DeepSeek: a remote provider
    assert!(
        screen.wait_for_text("API key (typing is hidden)", LIMIT),
        "{}",
        screen.plain_screen()
    );
    std::thread::sleep(SETTLE);
    screen.send(&format!("{SECRET}\n"));
    answer(&mut screen, "Step 2:", "\n");
    answer(&mut screen, "Step 3:", "n\n"); // never reach a real provider from a test
    assert!(
        screen.wait_for_text("not tested yet", LIMIT),
        "{}",
        screen.plain_screen()
    );
    assert_eq!(screen.finish(LIMIT), 0);
    assert!(sandbox.has_secret("DeepSeek"));
}

#[test]
fn the_typed_key_never_appears_on_the_screen() {
    let sandbox = Sandbox::new();
    let mut screen = terminal(&sandbox, None);
    answer(&mut screen, "Step 1:", "1\n");
    assert!(screen.wait_for_text("API key (typing is hidden)", LIMIT));
    std::thread::sleep(SETTLE);
    screen.send(&format!("{SECRET}\n"));
    answer(&mut screen, "Step 2:", "\n");
    answer(&mut screen, "Step 3:", "n\n");
    assert!(screen.wait_for_text("not tested yet", LIMIT));
    let everything = screen.screen();
    assert_eq!(screen.finish(LIMIT), 0);
    assert!(!everything.contains(SECRET), "the key was echoed");
}

#[test]
fn running_it_again_shows_the_menu_and_enter_leaves() {
    let server = serve(vec![]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let mut screen = terminal(&sandbox, None);
    assert!(
        screen.wait_for_text("What would you like to change?", LIMIT),
        "{}",
        screen.plain_screen()
    );
    assert!(screen.plain_screen().contains("\u{201c}Local\u{201d}"));
    screen.send("\n");
    assert_eq!(screen.finish(LIMIT), 0);
}

#[test]
fn a_missing_local_model_is_downloaded_with_ollama_only_after_a_yes() {
    // A stand-in `ollama` on PATH records how it was called; the "server" says it has no models.
    let bin = tempfile::tempdir().unwrap();
    let log = bin.path().join("calls.log");
    let program = bin.path().join("ollama");
    std::fs::write(
        &program,
        format!("#!/bin/sh\necho \"$@\" >> '{}'\n", log.display()),
    )
    .unwrap();
    make_executable(&program);
    let server = serve(vec![status_reply(200, r#"{"models":[]}"#), ok_reply("OK")]);
    let sandbox = Sandbox::new();
    let path = format!("{}:{}", bin.path().display(), sandbox.path_with_binary());
    let mut screen = terminal(&sandbox, Some(&path));
    answer(&mut screen, "Step 1:", "5\n");
    answer(&mut screen, "Name for this key", "Local\n");
    answer(&mut screen, "API address", &format!("{}\n", server.url));
    answer(&mut screen, "Model name:", "test-model\n");
    answer(&mut screen, "Download it now", "y\n");
    answer(&mut screen, "Step 2:", "\n");
    answer(&mut screen, "Step 3:", "y\n");
    assert!(
        screen.wait_for_text("All set", LIMIT),
        "{}",
        screen.plain_screen()
    );
    assert_eq!(screen.finish(LIMIT), 0);
    assert_eq!(
        std::fs::read_to_string(&log).unwrap().trim(),
        "pull test-model"
    );
}

#[test]
fn saying_no_to_the_download_never_starts_ollama() {
    let bin = tempfile::tempdir().unwrap();
    let log = bin.path().join("calls.log");
    let program = bin.path().join("ollama");
    std::fs::write(
        &program,
        format!("#!/bin/sh\necho \"$@\" >> '{}'\n", log.display()),
    )
    .unwrap();
    make_executable(&program);
    let server = serve(vec![status_reply(200, r#"{"models":[]}"#)]);
    let sandbox = Sandbox::new();
    let path = format!("{}:{}", bin.path().display(), sandbox.path_with_binary());
    let mut screen = terminal(&sandbox, Some(&path));
    answer(&mut screen, "Step 1:", "5\n");
    answer(&mut screen, "Name for this key", "Local\n");
    answer(&mut screen, "API address", &format!("{}\n", server.url));
    answer(&mut screen, "Model name:", "test-model\n");
    answer(&mut screen, "Download it now", "n\n");
    answer(&mut screen, "Step 2:", "\n");
    answer(&mut screen, "Step 3:", "n\n");
    assert!(screen.wait_for_text("not tested yet", LIMIT));
    assert_eq!(screen.finish(LIMIT), 0);
    assert!(!log.exists(), "ollama must not have been run");
}

fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}
