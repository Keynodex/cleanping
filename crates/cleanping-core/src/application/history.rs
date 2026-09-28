//! Look at, and delete, the local rewrite history.

use super::ports::RunRepository;
use crate::domain::errors::{CleanpingError, Result};
use crate::domain::models::Run;

pub struct HistoryService<R: RunRepository> {
    runs: R,
}

impl<R: RunRepository> HistoryService<R> {
    pub fn new(runs: R) -> Self {
        Self { runs }
    }

    /// The newest runs first, at most `limit`.
    pub fn recent(&self, limit: usize) -> Result<Vec<Run>> {
        self.runs.recent(limit)
    }

    /// Delete everything; returns how many runs were removed.
    pub fn clear(&self) -> Result<usize> {
        self.runs.delete_all()
    }

    /// Delete runs older than `cutoff` (a UTC stamp like 2026-09-28T18:30:00+00:00).
    pub fn purge_before(&self, cutoff: &str) -> Result<usize> {
        // Stamps compare as text, so a sloppy cutoff such as "z" would wipe everything.
        if !is_utc_stamp(cutoff) {
            return Err(CleanpingError::Validation(
                "That is not a valid date to delete before.".into(),
            ));
        }
        self.runs.delete_before(cutoff)
    }
}

/// `YYYY-MM-DDTHH:MM:SS+00:00`, the one format every run is stamped with.
fn is_utc_stamp(text: &str) -> bool {
    const SHAPE: &[u8; 25] = b"dddd-dd-ddTdd:dd:dd+00:00";
    text.len() == SHAPE.len()
        && text.bytes().zip(SHAPE).all(|(byte, shape)| match shape {
            b'd' => byte.is_ascii_digit(),
            other => byte == *other,
        })
}

#[cfg(test)]
#[path = "history_tests.rs"]
mod tests;
