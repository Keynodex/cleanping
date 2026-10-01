//! Finding the line of the reply that a command of the text became.
//!
//! First choice: the earliest unused reply line that starts with the same word (ignoring case),
//! a command line before any other line. Otherwise the unused line that shares the most words
//! with the command, if it shares at least two and at least half of them: the first word of a
//! line like `pls run: rm -r x/` is prose that a rewrite is expected to fix.

use std::collections::{HashMap, HashSet, VecDeque};

use super::command_lines::pieces;
use super::shell_words::scan;

/// How many reply lines the word-overlap search looks at, so a huge reply stays fast.
const SEARCHED: usize = 2_000;

/// The lines of a reply, each used at most once.
pub(super) struct Counterparts {
    lines: Vec<(String, HashSet<String>)>,
    used: Vec<bool>,
    by_first_word: HashMap<String, VecDeque<usize>>,
}

impl Counterparts {
    pub(super) fn of(reply: &str) -> Self {
        let mut all = pieces(reply);
        // Command lines first, so a first-word match prefers them; each group keeps its order.
        all.sort_by_key(|piece| !piece.command);
        let mut by_first_word: HashMap<String, VecDeque<usize>> = HashMap::new();
        let mut lines = Vec::with_capacity(all.len());
        for (at, piece) in all.into_iter().enumerate() {
            let words = words(&piece.text);
            if let Some(first) = first_word(&piece.text) {
                by_first_word.entry(first).or_default().push_back(at);
            }
            lines.push((piece.text, words));
        }
        Self {
            used: vec![false; lines.len()],
            lines,
            by_first_word,
        }
    }

    /// The reply line that `command` became, or `None` when no line is close enough.
    pub(super) fn take(&mut self, command: &str) -> Option<String> {
        let at = self
            .by_first(command)
            .or_else(|| self.by_overlap(command))?;
        self.used[at] = true;
        Some(self.lines[at].0.clone())
    }

    fn by_first(&mut self, command: &str) -> Option<usize> {
        let queue = self.by_first_word.get_mut(&first_word(command)?)?;
        while let Some(at) = queue.pop_front() {
            if !self.used[at] {
                return Some(at);
            }
        }
        None
    }

    fn by_overlap(&self, command: &str) -> Option<usize> {
        let wanted = words(command);
        let unused = (0..self.lines.len()).filter(|&at| !self.used[at]);
        let (at, shared) = unused
            .take(SEARCHED)
            .map(|at| (at, self.lines[at].1.intersection(&wanted).count()))
            .fold(
                None,
                |best: Option<(usize, usize)>, (at, shared)| match best {
                    Some((_, most)) if most >= shared => best,
                    _ => Some((at, shared)),
                },
            )?;
        (shared >= 2 && shared * 2 >= wanted.len()).then_some(at)
    }
}

fn first_word(text: &str) -> Option<String> {
    scan(text).words.first().map(|w| w.text.to_lowercase())
}

fn words(text: &str) -> HashSet<String> {
    scan(text).words.into_iter().map(|w| w.text).collect()
}
