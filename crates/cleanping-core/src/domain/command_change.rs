//! Spotting a command that a rewrite changed in a way that can hide a bug: a quote closed or
//! opened, a flag dropped or added, a path or URL changed. A pasted command with a missing quote
//! is often the very thing being asked about, and a model that "fixes" it hides the answer.
//!
//! A safety net, not a proof: it has false negatives and some false positives. Pure, no I/O.

use std::fmt;

use super::command_lines::pieces;
use super::command_match::{change, compare, label, Counterparts};
use super::shell_words::scan;

/// What happened to one command between the text and the reply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChangeKind {
    /// The text left a quote open (a shell would wait for more) and the reply closes it.
    QuoteClosed,
    /// The text's quotes were balanced and the reply leaves one open.
    QuoteOpened,
    /// This flag (shortened for display) is in the text's command but not in the reply's.
    FlagRemoved(String),
    /// This flag (shortened for display) is in the reply's command but not in the text's.
    FlagAdded(String),
    /// This path or URL (shortened for display) is in the text's command but not in the reply's.
    PathRemoved(String),
    /// This path or URL (shortened for display) is in the reply's command but not in the text's.
    PathAdded(String),
    /// No line of the reply starts with the command's first word.
    Missing,
}

/// One change to one command. `to_string()` says it in words that can be shown to a person.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandChange {
    /// The first words of the command in the text, at most 40 characters; never a secret.
    pub command: String,
    /// What changed.
    pub kind: ChangeKind,
}

impl fmt::Display for CommandChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let command = &self.command;
        match &self.kind {
            ChangeKind::QuoteClosed => {
                write!(f, "a quote was closed in the command starting `{command}`")
            }
            ChangeKind::QuoteOpened => {
                write!(f, "a quote was left open in the command starting `{command}`")
            }
            ChangeKind::FlagRemoved(flag) => write!(
                f,
                "the flag `{flag}` was removed from the command starting `{command}`"
            ),
            ChangeKind::FlagAdded(flag) => write!(
                f,
                "the flag `{flag}` was added to the command starting `{command}`"
            ),
            ChangeKind::PathRemoved(path) => write!(
                f,
                "the path or URL `{path}` was changed or removed in the command starting `{command}`"
            ),
            ChangeKind::PathAdded(path) => write!(
                f,
                "the path or URL `{path}` was added to the command starting `{command}`"
            ),
            ChangeKind::Missing => {
                write!(f, "the command starting `{command}` is no longer in the reply")
            }
        }
    }
}

/// Changes to the commands in a text that mixes prose and commands (see
/// [`command_lines`](super::command_lines) for which lines count as commands). Prose is ignored.
/// Each command of the text is compared with the next line of the reply that starts with the
/// same word (ignoring case), a command line first, else any line.
pub fn command_changes(original: &str, reply: &str) -> Vec<CommandChange> {
    let mut counterparts = Counterparts::of(reply);
    pieces(original)
        .into_iter()
        .filter(|piece| piece.command)
        .flat_map(|piece| match counterparts.take(&piece.text) {
            Some(after) => compare(&piece.text, &after),
            None => vec![change(&label(&scan(&piece.text)), ChangeKind::Missing)],
        })
        .collect()
}

/// Changes when the whole text is one command, as on a shell's command line. Blank text has no
/// command, so nothing can change.
pub fn command_line_changes(original: &str, reply: &str) -> Vec<CommandChange> {
    if original.trim().is_empty() {
        return Vec::new();
    }
    compare(original.trim(), reply.trim())
}

/// The first change in words, with how many more there are, or `None` when there are none.
pub fn summary(changes: &[CommandChange]) -> Option<String> {
    let first = changes.first()?;
    Some(match changes.len() - 1 {
        0 => first.to_string(),
        more => format!("{first} (and {more} more)"),
    })
}

#[cfg(test)]
#[path = "command_change_tests.rs"]
mod tests;
