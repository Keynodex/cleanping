//! Editor history: the user's text and every rewrite of it.
//!
//! Not used by the command line. It is kept, with its Python-compatible saved form, for the
//! desktop window that will be ported next.

use serde_json::{json, Value};

/// Adding a version to a stack that already holds this many drops the oldest one.
pub const MAX_VERSIONS: usize = 20;

/// The user's text and its rewrites, oldest first, with a position on the one shown. It always
/// holds at least one version, and the position always points at one of them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionStack {
    versions: Vec<String>,
    index: usize,
}

impl Default for VersionStack {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionStack {
    /// A stack with one empty version.
    pub fn new() -> Self {
        Self {
            versions: vec![String::new()],
            index: 0,
        }
    }

    /// Empty `versions` gives a fresh stack; `index` is clamped, `None` means the latest.
    pub fn from_parts(versions: Vec<String>, index: Option<usize>) -> Self {
        if versions.is_empty() {
            return Self::new();
        }
        let last = versions.len() - 1;
        Self {
            index: index.map_or(last, |i| i.min(last)),
            versions,
        }
    }

    /// Every kept version, oldest first.
    pub fn versions(&self) -> &[String] {
        &self.versions
    }

    /// The version shown now.
    pub fn current(&self) -> &str {
        &self.versions[self.index]
    }

    /// Position of the shown version; 0 is the oldest.
    pub fn index(&self) -> usize {
        self.index
    }

    /// How many versions are kept (never 0).
    pub fn count(&self) -> usize {
        self.versions.len()
    }

    /// The user changed the shown text: update the latest, or fork from an older one.
    pub fn edit(&mut self, text: &str) {
        if text == self.current() {
            return;
        }
        if self.index == self.versions.len() - 1 {
            self.versions[self.index] = text.to_string();
        } else {
            self.append(text);
        }
    }

    /// Add a rewrite as the new latest. False when it equals what is already shown.
    pub fn push(&mut self, text: &str) -> bool {
        if text == self.current() && self.index == self.versions.len() - 1 {
            return false;
        }
        self.append(text);
        true
    }

    /// Move to the next older version; false (and no change) when already at the oldest.
    pub fn back(&mut self) -> bool {
        self.index
            .checked_sub(1)
            .is_some_and(|target| self.move_to(target))
    }

    /// Move to the next newer version; false (and no change) when already at the latest.
    pub fn forward(&mut self) -> bool {
        self.move_to(self.index + 1)
    }

    /// Flip between the oldest kept version and the latest.
    pub fn toggle_original(&mut self) -> bool {
        if self.versions.len() < 2 {
            return false;
        }
        self.move_to(if self.index == 0 {
            self.versions.len() - 1
        } else {
            0
        })
    }

    /// The saved form: JSON with `versions` and `index`, as the Python release wrote it. Read it
    /// back with [`loads`](Self::loads).
    pub fn dumps(&self) -> String {
        json!({ "versions": self.versions, "index": self.index }).to_string()
    }

    /// Tolerant of corrupt input: anything unusable becomes a fresh stack.
    pub fn loads(raw: &str) -> Self {
        let Ok(value) = serde_json::from_str::<Value>(raw) else {
            return Self::new();
        };
        let versions: Option<Vec<String>> = value
            .get("versions")
            .and_then(Value::as_array)
            .and_then(|items| {
                items
                    .iter()
                    .map(|v| v.as_str().map(str::to_owned))
                    .collect()
            });
        let index = value
            .get("index")
            .and_then(Value::as_i64)
            .map(|i| usize::try_from(i.max(0)).unwrap_or(usize::MAX));
        versions.map_or_else(Self::new, |v| Self::from_parts(v, index))
    }

    fn append(&mut self, text: &str) {
        self.versions.push(text.to_string());
        if self.versions.len() > MAX_VERSIONS {
            self.versions.remove(0);
        }
        self.index = self.versions.len() - 1;
    }

    fn move_to(&mut self, target: usize) -> bool {
        if target >= self.versions.len() || target == self.index {
            return false;
        }
        self.index = target;
        true
    }
}

#[cfg(test)]
#[path = "versions_tests.rs"]
mod tests;
