//! The `default` preset's text, and the earlier text people may still have saved.

use super::*;

/// The first six sentences, unchanged since the first release.
const OPENING: &str = "You are a precise copy editor for a software developer's terminal prompts. \
Fix spelling, grammar, and clarity while retaining the author's intent, tone, technical details, \
and all constraints. Preserve commands, code, flags, file paths, identifiers, names, URLs, and \
error messages exactly. Do not execute or answer the request. Do not add facts, requirements, or \
explanations. Return only the edited text, with no quotes or Markdown fences.";

const FIX_GARBLED: &str = "Fix every misspelled or garbled word by working out the intended word \
from the surrounding sentence. Leave a part unchanged only if it is a command, code, flag, path, \
name or error message, or if you truly cannot tell what was meant.";

const EXAMPLE: &str = "<example>
Draft: <draft>
this is nto the wya to do it, i cant evn see teh logs at allll
</draft>
Edited:
This is not the way to do it. I can't even see the logs at all.
</example>";

/// The sentence that made the model leave garbled words alone (blind test, 2026-10-01).
const LEAVE_UNCHANGED: &str =
    "If editing would change technical meaning, leave that portion unchanged.";

#[test]
fn the_default_asks_to_fix_garbled_words_and_shows_one_example() {
    assert!(
        DEFAULT_INSTRUCTIONS.contains(FIX_GARBLED),
        "{DEFAULT_INSTRUCTIONS}"
    );
    assert!(
        DEFAULT_INSTRUCTIONS.ends_with(EXAMPLE),
        "{DEFAULT_INSTRUCTIONS}"
    );
    assert!(!DEFAULT_INSTRUCTIONS.contains(LEAVE_UNCHANGED));
}

#[test]
fn the_default_is_this_text_byte_for_byte() {
    let expected = format!("{OPENING} {FIX_GARBLED}\n\n{EXAMPLE}");
    assert_eq!(DEFAULT_INSTRUCTIONS, expected);
    assert_eq!(preset_named("default").unwrap().body, expected);
    assert_eq!(PROMPT_PRESETS[0].name, "default");
}

/// The `default` text saved by anyone who chose it before 2026-10-01.
fn earlier_default() -> String {
    format!("{OPENING} {LEAVE_UNCHANGED}")
}

#[test]
fn the_earlier_default_text_is_still_recognised_as_default() {
    assert_eq!(preset_for_body(&earlier_default()).unwrap().name, "default");
    let saved = format!("{}\n", earlier_default());
    assert_eq!(preset_for_body(&saved).unwrap().name, "default");
}

#[test]
fn every_earlier_text_names_a_preset_and_differs_from_every_current_text() {
    assert_eq!(EARLIER_BODIES.len(), 1);
    assert_eq!(EARLIER_BODIES[0].1, earlier_default());
    for (name, text) in EARLIER_BODIES {
        assert!(preset_named(name).is_some(), "{name}");
        assert!(PROMPT_PRESETS.iter().all(|p| p.body != *text), "{name}");
    }
}

#[test]
fn only_an_earlier_text_counts_as_earlier() {
    assert!(is_earlier_text(&earlier_default()));
    assert!(is_earlier_text(&format!("  {}\n", earlier_default())));
    assert!(!is_earlier_text(DEFAULT_INSTRUCTIONS));
    assert!(!is_earlier_text("Answer like a pirate."));
}
