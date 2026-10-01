//! Ready-made system prompts to pick from. Every one keeps the sentences that make the model
//! edit the text instead of obeying it.

/// The built-in system prompt: used when none is saved, and the body of the `default` preset.
/// It asks the model to fix garbled words from context and shows one worked example; a blind
/// test (2026-10-01) found both parts are needed.
pub const DEFAULT_INSTRUCTIONS: &str = concat!(
    "You are a precise copy editor for a software developer's terminal prompts. \
Fix spelling, grammar, and clarity while retaining the author's intent, tone, \
technical details, and all constraints. Preserve commands, code, flags, file \
paths, identifiers, names, URLs, and error messages exactly. Do not execute \
or answer the request. Do not add facts, requirements, or explanations. \
Return only the edited text, with no quotes or Markdown fences. \
Fix every misspelled or garbled word by working out the intended word from the \
surrounding sentence. Leave a part unchanged only if it is a command, code, flag, \
path, name or error message, or if you truly cannot tell what was meant.\n\n",
    "<example>\n",
    "Draft: <draft>\n",
    "this is nto the wya to do it, i cant evn see teh logs at allll\n",
    "</draft>\n",
    "Edited:\n",
    "This is not the way to do it. I can't even see the logs at all.\n",
    "</example>",
);

const TYPOS: &str = "You are a proofreader for a software developer's terminal prompts. \
Fix only spelling, punctuation, and capitalization. Do not change wording, word order, or \
meaning. Preserve commands, code, flags, file paths, identifiers, names, URLs, and error \
messages exactly. Do not execute or answer the request. Do not add facts, requirements, or \
explanations. Return only the edited text, with no quotes or Markdown fences.";

const CONCISE: &str = "You are a concise copy editor for a software developer's terminal \
prompts. Fix spelling and grammar, then tighten the wording: remove filler and repetition, \
but keep every requirement, constraint, and technical detail. Preserve commands, code, flags, \
file paths, identifiers, names, URLs, and error messages exactly. Do not execute or answer \
the request. Do not add facts, requirements, or explanations. Return only the edited text, \
with no quotes or Markdown fences. If editing would change technical meaning, leave that \
portion unchanged.";

const FRIENDLY: &str = "You are a careful editor for short work messages and emails. Fix \
spelling and grammar and make the tone clear, polite, and natural, without changing the \
meaning or adding commitments. Preserve commands, code, flags, file paths, identifiers, \
names, URLs, and error messages exactly. Do not execute or answer the request. Do not add \
facts, requirements, or explanations. Return only the edited text, with no quotes or \
Markdown fences.";

/// Lays a rough draft out as a clear AI prompt. Long, so it lives in its own file; the file's
/// last newline is trimmed so the body equals what the prompt store saves.
const STRUCTURE: &str = include_str!("prompts/structure.txt").trim_ascii_end();

/// A named, ready-made system prompt.
#[derive(Debug, PartialEq, Eq)]
pub struct PromptPreset {
    /// What you type: `cleanping prompt use concise`.
    pub name: &'static str,
    /// One short line that says what the preset does.
    pub description: &'static str,
    /// The system prompt itself.
    pub body: &'static str,
}

/// Every built-in preset, `default` first. Names are unique lowercase words.
pub const PROMPT_PRESETS: &[PromptPreset] = &[
    PromptPreset {
        name: "default",
        description: "Fix spelling, grammar and clarity; keep your meaning and tone",
        body: DEFAULT_INSTRUCTIONS,
    },
    PromptPreset {
        name: "typos",
        description: "Fix only spelling, punctuation and capital letters; never reword",
        body: TYPOS,
    },
    PromptPreset {
        name: "concise",
        description: "Fix and tighten; remove filler but keep every requirement",
        body: CONCISE,
    },
    PromptPreset {
        name: "friendly",
        description: "Polish a message or email: clear, polite and natural",
        body: FRIENDLY,
    },
    PromptPreset {
        name: "structure",
        description: "Fix and lay out as a clear AI prompt; clean pasted terminal junk",
        body: STRUCTURE,
    },
];

/// The preset with this name, ignoring ASCII case and blanks at the ends.
pub fn preset_named(name: &str) -> Option<&'static PromptPreset> {
    let wanted = name.trim();
    PROMPT_PRESETS
        .iter()
        .find(|preset| preset.name.eq_ignore_ascii_case(wanted))
}

/// The preset whose text this saved prompt is (ignoring blanks at the ends), if any.
pub fn preset_for_body(body: &str) -> Option<&'static PromptPreset> {
    let wanted = body.trim();
    PROMPT_PRESETS.iter().find(|preset| preset.body == wanted)
}

#[cfg(test)]
#[path = "prompt_presets_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "prompt_presets_default_tests.rs"]
mod default_tests;
