//! `cleanping writer`: a text-only terminal mode. Drives the real binary inside a pseudo-terminal
//! (Linux `script`) and waits for what it prints, never for a fixed time.
#![cfg(target_os = "linux")]

mod support;

use std::path::Path;

use support::shell::available;
use support::writer::*;
use support::*;

#[test]
fn without_a_terminal_it_says_so_and_exits_2() {
    let out = Sandbox::new().run(&["writer"], None);
    assert_eq!(
        out.code, 2,
        "writer without a terminal must exit 2: {}",
        out.stderr
    );
    assert!(out.stderr.contains("needs a terminal"), "{}", out.stderr);
    assert_eq!(out.stdout, "");
}

#[test]
fn with_its_output_redirected_it_also_needs_a_terminal() {
    let sandbox = Sandbox::new();
    let screen = run_in_terminal(
        &sandbox,
        &sandbox.path_with_binary(),
        &[],
        "cleanping writer > /dev/null; echo code=$?",
    );
    assert!(
        screen.wait_for_text("code=2", LIMIT),
        "redirected output must exit 2: {}",
        screen.plain_screen()
    );
    assert!(screen.plain_screen().contains("needs a terminal"));
}

#[test]
fn text_that_starts_with_writer_needs_a_double_dash_like_edit_and_setup() {
    let server = serve(vec![ok_reply("Writer notes.")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    let out = sandbox.run(&["--", "writer", "notes"], None);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert_eq!(out.stdout, "Writer notes.\n");
    assert_eq!(server.user_text(0), "writer notes");
    let bare = sandbox.run(&["writer", "notes"], None);
    assert_eq!(
        bare.code, 2,
        "writer is a command word, so extra words are refused"
    );
    assert_eq!(
        server.requests.lock().unwrap().len(),
        1,
        "nothing more was sent"
    );
}

#[test]
fn without_zsh_it_says_zsh_is_required_and_exits_1() {
    if !available("zsh") {
        return;
    }
    let sandbox = Sandbox::new();
    let bare = bare_dir(&sandbox, &["script"]);
    let binary = sandbox.path_with_binary();
    let binary_dir = binary.split(':').next().unwrap();
    let path = format!("{binary_dir}:{}", bare.display());
    let screen = start(&sandbox, &path, &[]);
    assert!(
        screen.wait_for_text("needs zsh", LIMIT),
        "a missing zsh must be explained: {}",
        screen.plain_screen()
    );
    assert_eq!(screen.finish(LIMIT), 1);
}

#[test]
fn it_starts_with_a_short_banner_and_a_plain_prompt() {
    if !available("zsh") {
        return;
    }
    let sandbox = Sandbox::new();
    let screen = start(&sandbox, &sandbox.path_with_binary(), &[]);
    wait_for_prompt(&screen);
    let shown = screen.plain_screen();
    for line in [
        "CleanPing writer",
        "Type your text. Press Ctrl+G to fix it. Press Enter to copy it. Press Ctrl+D to leave.",
        "Nothing you type here is ever run.",
    ] {
        assert!(shown.contains(line), "missing {line:?} in: {shown}");
    }
}

#[test]
fn ctrl_d_on_an_empty_line_leaves_with_status_0() {
    if !available("zsh") {
        return;
    }
    let sandbox = Sandbox::new();
    let mut screen = start(&sandbox, &sandbox.path_with_binary(), &[]);
    wait_for_prompt(&screen);
    screen.send(EOF);
    let code = screen.wait_for_exit(LIMIT);
    assert_eq!(
        code,
        Some(0),
        "Ctrl+D must leave: {}",
        screen.plain_screen()
    );
}

fn write_canary(path: &Path, marker: &Path) {
    std::fs::write(
        path,
        format!("echo {} >> '{}'\n", path.display(), marker.display()),
    )
    .unwrap();
}

#[test]
fn none_of_the_users_startup_files_are_read() {
    if !available("zsh") {
        return;
    }
    let sandbox = Sandbox::new();
    let home = sandbox.dir.path();
    let zdotdir = home.join("zdot");
    std::fs::create_dir_all(&zdotdir).unwrap();
    let marker = home.join("marker.txt");
    for name in [".zshenv", ".zprofile", ".zshrc", ".zlogin"] {
        write_canary(&home.join(name), &marker);
        write_canary(&zdotdir.join(name), &marker);
    }
    let zdot = zdotdir.to_str().unwrap();
    // Positive control: the same canaries do run for an ordinary interactive zsh.
    let path = sandbox.path_with_binary();
    let control = run_in_terminal(&sandbox, &path, &[("ZDOTDIR", zdot)], "zsh -i -c true");
    assert_eq!(control.finish(LIMIT), 0);
    assert!(
        marker.exists(),
        "the canary files do not work, so this test proves nothing"
    );
    std::fs::remove_file(&marker).unwrap();

    for extra in [&[][..], &[("ZDOTDIR", zdot)][..]] {
        let mut screen = start(&sandbox, &path, extra);
        wait_for_prompt(&screen);
        screen.send(EOF);
        assert_eq!(screen.finish(LIMIT), 0);
        assert!(!marker.exists(), "a startup file of the user was read");
    }
}

#[test]
fn the_private_startup_file_is_owner_only_and_made_fresh_each_run() {
    use std::os::unix::fs::PermissionsExt;
    if !available("zsh") {
        return;
    }
    let sandbox = Sandbox::new();
    let dir = sandbox.dir.path().join("data/cleanping/writer");
    let rc = dir.join(".zshrc");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::write(&rc, "echo STALE\n").unwrap();
    std::fs::set_permissions(&rc, std::fs::Permissions::from_mode(0o644)).unwrap();
    let mut screen = start(&sandbox, &sandbox.path_with_binary(), &[]);
    wait_for_prompt(&screen);
    let mode = |path: &Path| std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
    assert_eq!((mode(&dir), mode(&rc)), (0o700, 0o600));
    let text = std::fs::read_to_string(&rc).unwrap();
    assert!(
        text.contains("CleanPing writer") && !text.contains("STALE"),
        "{text}"
    );
    screen.send(EOF);
    screen.finish(LIMIT);
}
