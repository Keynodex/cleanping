//! Pairing a command of the text with its line in the reply, and comparing the two.

use std::collections::{HashMap, HashSet, VecDeque};

use super::command_change::{ChangeKind, CommandChange};
use super::command_lines::{flag, path, pieces};
use super::secret_scan::find_secret;
use super::shell_words::{scan, Scan, Word};

/// How many characters of a word or command are shown.
const SHOWN_CHARS: usize = 40;
/// How many words of a command name it in a message.
const LABEL_WORDS: usize = 3;

/// The reply's lines, by their first word: command lines first, then any other line.
pub(super) struct Counterparts {
    by_word: HashMap<String, (VecDeque<String>, VecDeque<String>)>,
}

impl Counterparts {
    pub(super) fn of(reply: &str) -> Self {
        let mut by_word: HashMap<_, (VecDeque<_>, VecDeque<_>)> = HashMap::new();
        for piece in pieces(reply) {
            let Some(key) = first_word(&piece.text) else {
                continue;
            };
            let (commands, others) = by_word.entry(key).or_default();
            if piece.command {
                commands.push_back(piece.text);
            } else {
                others.push_back(piece.text);
            }
        }
        Self { by_word }
    }

    /// The earliest unused line of the reply that starts with the same word as `command`.
    pub(super) fn take(&mut self, command: &str) -> Option<String> {
        let (commands, others) = self.by_word.get_mut(&first_word(command)?)?;
        commands.pop_front().or_else(|| others.pop_front())
    }
}

fn first_word(text: &str) -> Option<String> {
    scan(text).words.first().map(|w| w.text.to_lowercase())
}

/// Every change from the command `before` to the command `after`.
pub(super) fn compare(before: &str, after: &str) -> Vec<CommandChange> {
    let (before, after) = (scan(before), scan(after));
    let mut kinds = Vec::new();
    match (before.open.is_some(), after.open.is_some()) {
        (true, false) => kinds.push(ChangeKind::QuoteClosed),
        (false, true) => kinds.push(ChangeKind::QuoteOpened),
        _ => {}
    }
    let (gone, new) = differences(&before, &after, flag);
    kinds.extend(gone.into_iter().map(ChangeKind::FlagRemoved));
    kinds.extend(new.into_iter().map(ChangeKind::FlagAdded));
    let (gone, new) = differences(&before, &after, path);
    kinds.extend(gone.into_iter().map(ChangeKind::PathRemoved));
    kinds.extend(new.into_iter().map(ChangeKind::PathAdded));
    let label = label(&before);
    kinds.into_iter().map(|kind| change(&label, kind)).collect()
}

type Pick = fn(&Word) -> Option<&str>;

/// The picked words only in `before`, and those only in `after`, in order, shown safely.
fn differences(before: &Scan, after: &Scan, pick: Pick) -> (Vec<String>, Vec<String>) {
    let picked = |scan: &Scan| -> Vec<String> {
        let mut seen = HashSet::new();
        let words = scan.words.iter().filter_map(pick);
        words
            .filter(|w| seen.insert(*w))
            .map(String::from)
            .collect()
    };
    let (before, after) = (picked(before), picked(after));
    let only = |these: &[String], those: &[String]| -> Vec<String> {
        let those: HashSet<&String> = those.iter().collect();
        these
            .iter()
            .filter(|w| !those.contains(w))
            .map(|w| shown(w))
            .collect()
    };
    (only(&before, &after), only(&after, &before))
}

pub(super) fn change(label: &str, kind: ChangeKind) -> CommandChange {
    CommandChange {
        command: label.to_string(),
        kind,
    }
}

/// The first few words of a command, as it is named in a message.
pub(super) fn label(scan: &Scan) -> String {
    let words: Vec<&str> = scan
        .words
        .iter()
        .take(LABEL_WORDS)
        .map(|w| &w.text[..])
        .collect();
    shown(&words.join(" "))
}

/// `text` as it may be shown: at most 40 characters, and never something that looks like a
/// secret (checked on the whole text, before it is shortened).
fn shown(text: &str) -> String {
    if find_secret(text).is_some() {
        return "(hidden: looks like a secret)".into();
    }
    let text = text.replace(['\n', '\r', '\t'], " ");
    if text.chars().count() <= SHOWN_CHARS {
        return text;
    }
    let mut short: String = text.chars().take(SHOWN_CHARS - 1).collect();
    short.push('\u{2026}');
    short
}
