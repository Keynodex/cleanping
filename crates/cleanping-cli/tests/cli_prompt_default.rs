//! The `default` system prompt through the real binary, including for someone who saved the
//! earlier `default` text (before 2026-10-01).

mod support;

use support::*;

/// The `default` text before 2026-10-01, as anyone who chose it then has it saved.
const EARLIER_DEFAULT: &str = "You are a precise copy editor for a software developer's terminal \
prompts. Fix spelling, grammar, and clarity while retaining the author's intent, tone, technical \
details, and all constraints. Preserve commands, code, flags, file paths, identifiers, names, \
URLs, and error messages exactly. Do not execute or answer the request. Do not add facts, \
requirements, or explanations. Return only the edited text, with no quotes or Markdown fences. \
If editing would change technical meaning, leave that portion unchanged.";

const NEW_SENTENCE: &str = "Fix every misspelled or garbled word";

const UPDATE_HINT: &str = "\nYou have an earlier version of default. \
Get the current one with: cleanping prompt use default\n";

fn marked(listed: &str) -> Vec<&str> {
    listed.lines().filter(|l| l.starts_with('*')).collect()
}

fn on_the_earlier_default() -> Sandbox {
    let sandbox = Sandbox::new();
    let saved = sandbox.run(&["prompt", "set"], Some(EARLIER_DEFAULT));
    assert_eq!(saved.code, 0, "{}", saved.stderr);
    sandbox
}

#[test]
fn prompt_show_prints_the_new_default_with_its_example() {
    let shown = Sandbox::new().run(&["prompt", "show"], None).stdout;
    assert!(shown.contains(NEW_SENTENCE), "{shown}");
    assert!(shown.ends_with("</example>\n"), "{shown}");
    assert!(!shown.contains("leave that portion unchanged"), "{shown}");
}

#[test]
fn the_earlier_default_is_marked_as_default() {
    let sandbox = on_the_earlier_default();
    let listed = sandbox.run(&["prompt", "presets"], None);
    assert_eq!(listed.code, 0, "{}", listed.stderr);
    let marked = marked(&listed.stdout);
    assert_eq!(marked.len(), 1, "{}", listed.stdout);
    assert!(marked[0].starts_with("* default "), "{}", listed.stdout);
    assert!(
        listed.stdout.ends_with(UPDATE_HINT),
        "no hint in: {}",
        listed.stdout
    );
}

#[test]
fn the_current_default_gets_no_hint() {
    let listed = Sandbox::new().run(&["prompt", "presets"], None).stdout;
    assert!(marked(&listed)[0].starts_with("* default "), "{listed}");
    assert!(!listed.contains("earlier version"), "{listed}");
}

#[test]
fn prompt_use_default_replaces_the_earlier_default_without_yes() {
    let sandbox = on_the_earlier_default();
    let used = sandbox.run(&["prompt", "use", "default"], None);
    assert_eq!(used.code, 0, "{}", used.stderr);
    let shown = sandbox.run(&["prompt", "show"], None).stdout;
    assert!(shown.contains(NEW_SENTENCE), "{shown}");
}

#[test]
fn prompt_use_default_still_needs_yes_to_replace_your_own_prompt() {
    let sandbox = Sandbox::new();
    let own = "Answer like a pirate.";
    assert_eq!(sandbox.run(&["prompt", "set", own], None).code, 0);
    let refused = sandbox.run(&["prompt", "use", "default"], None);
    assert_eq!(refused.code, 2, "{}", refused.stderr);
    assert!(refused.stderr.contains("--yes"), "{}", refused.stderr);
    assert_eq!(sandbox.run(&["prompt", "show"], None).stdout.trim(), own);
    let replaced = sandbox.run(&["prompt", "use", "default", "--yes"], None);
    assert_eq!(replaced.code, 0, "{}", replaced.stderr);
    let shown = sandbox.run(&["prompt", "show"], None).stdout;
    assert!(shown.contains(NEW_SENTENCE), "{shown}");
}
