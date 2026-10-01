//! When a reply changes a command in the text: the plain command warns on stderr and still
//! prints the reply; `--keep-shape` (the shell key) refuses it.

mod support;

use support::*;

const ASKED: &str = "why is it waiting?\n```\ncurl -d '{\"name\": \"lamp\"}\n```";
const CLOSED: &str = "Why is it waiting?\n```\ncurl -d '{\"name\": \"lamp\"}'\n```";
const WARNING: &str =
    "cleanping: warning: a quote was closed in the command starting `curl -d '{\"name\": \"lamp\"}`.";

fn with_reply(reply: &str) -> Sandbox {
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &serve(vec![ok_reply(reply)]));
    sandbox
}

#[test]
fn a_closed_quote_is_a_warning_and_the_reply_is_still_printed() {
    let out = with_reply(CLOSED).run(&[], Some(ASKED));
    assert_eq!(out.code, 0, "the exit code does not change: {}", out.stderr);
    assert_eq!(out.stdout, format!("{CLOSED}\n"));
    assert_eq!(out.stderr.trim_end(), WARNING);
}

#[test]
fn text_given_as_arguments_is_checked_the_same_way() {
    let out = with_reply("Please run: rm -r build/").run(&["pls run: rm -r -f build/"], None);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(
        out.stderr
            .contains("warning: the flag `-f` was removed from the command starting `pls run: rm`"),
        "{}",
        out.stderr
    );
}

#[test]
fn a_reply_that_leaves_the_command_alone_has_no_warning() {
    let reply = "Why is it waiting?\n```\ncurl -d '{\"name\": \"lamp\"}\n```";
    let out = with_reply(reply).run(&[], Some(ASKED));
    assert_eq!((out.code, out.stderr.as_str()), (0, ""));
    assert_eq!(out.stdout, format!("{reply}\n"));
}

#[test]
fn keep_shape_refuses_a_reply_that_changes_the_command_line() {
    let typed = "curl -d '{\"name\": \"lamp\"}";
    let out = with_reply("curl -d '{\"name\": \"lamp\"}'").run(&["--keep-shape"], Some(typed));
    assert_eq!((out.code, out.stdout.as_str()), (1, ""), "{}", out.stderr);
    assert!(
        out.stderr
            .contains("The reply changed your command (a quote was closed"),
        "{}",
        out.stderr
    );
    assert!(out.stderr.contains("not applied"), "{}", out.stderr);
}

#[test]
fn keep_shape_still_applies_a_reply_that_keeps_the_command() {
    let out = with_reply("git commit -m \"Fix the login\"")
        .run(&["--keep-shape"], Some("git commit -m \"fix teh login\""));
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert_eq!(out.stdout, "git commit -m \"Fix the login\"\n");
}
