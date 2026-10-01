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
    assert!(DEFAULT_INSTRUCTIONS.contains(FIX_GARBLED), "{DEFAULT_INSTRUCTIONS}");
    assert!(DEFAULT_INSTRUCTIONS.ends_with(EXAMPLE), "{DEFAULT_INSTRUCTIONS}");
    assert!(!DEFAULT_INSTRUCTIONS.contains(LEAVE_UNCHANGED));
}

#[test]
fn the_default_is_this_text_byte_for_byte() {
    let expected = format!("{OPENING} {FIX_GARBLED}\n\n{EXAMPLE}");
    assert_eq!(DEFAULT_INSTRUCTIONS, expected);
    assert_eq!(preset_named("default").unwrap().body, expected);
    assert_eq!(PROMPT_PRESETS[0].name, "default");
}
