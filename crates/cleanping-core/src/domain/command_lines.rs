//! Which lines of a text are commands. A line is a command when it is
//!
//! 1. inside a fenced block (a line starting with three backticks opens and closes it),
//! 2. after a `$ ` prompt marker,
//! 3. indented by 4 or more spaces or a tab and it looks like a command, or
//! 4. not indented that far, starts with a plain word and has a flag (`-x`, `--name`) after it.
//!
//! "Looks like a command" means the first word is plain (letters, digits, `.`, `_`, `-`, `/`,
//! `~`, `+`, not starting with `-`) and a flag or a path follows. A command goes on over the next
//! lines while it ends in a backslash or leaves a quote open: inside a fence every next line
//! counts, outside one only a line starting with `> ` (a shell's continuation prompt).

use super::shell_words::{scan, Scanner, Word};

/// A command or a plain line of a text, with the markers (`$ `, `> `, indentation) removed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    /// The command, its continuation lines joined with newlines.
    pub text: String,
    /// True when the rules above found a command here; false for any other non-blank line.
    pub command: bool,
}

/// Every command and every other non-blank line of `text`, in order. Fence lines are left out.
pub fn pieces(text: &str) -> Vec<Piece> {
    let mut lines = text.lines().peekable();
    let mut fenced = false;
    let mut found = Vec::new();
    while let Some(line) = lines.next() {
        if is_fence(line) {
            fenced = !fenced;
            continue;
        }
        let Some(start) = command_start(line, fenced) else {
            if !line.trim().is_empty() {
                let plain = line.trim();
                let plain = plain.strip_prefix("$ ").unwrap_or(plain);
                found.push(piece(plain, false));
            }
            continue;
        };
        let mut scanner = Scanner::default();
        let mut command = start.to_string();
        scanner.feed(start);
        while scanner.continues() {
            let Some(next) = lines.peek().and_then(|l| continuation(l, fenced)) else {
                break;
            };
            command.push('\n');
            command.push_str(next);
            scanner.feed("\n");
            scanner.feed(next);
            lines.next();
        }
        found.push(piece(&command, true));
    }
    found
}

fn piece(text: &str, command: bool) -> Piece {
    Piece {
        text: text.to_string(),
        command,
    }
}

fn is_fence(line: &str) -> bool {
    line.trim_start().starts_with("```")
}

fn command_start(line: &str, fenced: bool) -> Option<&str> {
    let trimmed = line.trim_start();
    if trimmed.trim_end().is_empty() {
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix("$ ") {
        return Some(rest);
    }
    let indented = line.starts_with('\t') || line.starts_with("    ");
    let command = fenced
        || (indented && looks_like_command(trimmed, true))
        || (!indented && looks_like_command(trimmed, false));
    command.then_some(trimmed)
}

fn continuation(line: &str, fenced: bool) -> Option<&str> {
    if is_fence(line) {
        return None;
    }
    let marked = line.strip_prefix("> ").or((line == ">").then_some(""));
    if fenced {
        return Some(marked.unwrap_or(line));
    }
    marked
}

/// A plain first word followed by a flag, or by a path when `paths_count`.
fn looks_like_command(line: &str, paths_count: bool) -> bool {
    let words = scan(line).words;
    let Some((first, rest)) = words.split_first() else {
        return false;
    };
    is_plain(first)
        && rest
            .iter()
            .any(|w| flag(w).is_some() || (paths_count && path(w).is_some()))
}

fn is_plain(word: &Word) -> bool {
    let text = &word.text;
    !word.quoted
        && !text.starts_with('-')
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-/~+".contains(c))
}

/// Punctuation a sentence may put after a word, which is not part of it.
const TRAILING: &[char] = &['.', ',', ';', ':', '!', '?', ')'];

/// `-x`, `-la`, `--name` or `--name=value`, without sentence punctuation after it.
pub fn flag(word: &Word) -> Option<&str> {
    let text = word.text.trim_end_matches(TRAILING);
    let body = text.strip_prefix("--").or_else(|| text.strip_prefix('-'))?;
    let starts_well = body.chars().next().is_some_and(char::is_alphanumeric);
    (!word.quoted && starts_well).then_some(text)
}

/// A word with a `/` or starting with `~` (a path or a URL), without sentence punctuation after
/// it. Words in quotes and flags are not paths here.
pub fn path(word: &Word) -> Option<&str> {
    let text = word.text.trim_end_matches(TRAILING);
    let shaped = text.contains('/') || text.starts_with('~');
    let has_name = text.chars().any(char::is_alphanumeric);
    (!word.quoted && shaped && has_name && flag(word).is_none()).then_some(text)
}

#[cfg(test)]
#[path = "command_lines_tests.rs"]
mod tests;
