//! `--marks`: which parts of a reply changed, for the shell key to highlight. Pure text in,
//! text out: no database, no files, no network, so it is safe to run on every key press.

use cleanping_core::domain::diff::highlight_changes;
use cleanping_core::domain::errors::{CleanpingError, Result};

use crate::input;
use crate::output;

/// Stdin holds the original text, a NUL byte, then the reply. Prints one `start end` line per
/// changed range of the reply, in characters. Prints nothing when the texts are too large to
/// compare quickly, so the caller simply shows no highlight.
pub fn run() -> Result<()> {
    let both = input::read_stdin()?;
    let (original, reply) = both.split_once('\0').ok_or_else(|| {
        CleanpingError::Validation("Expected the original text, a NUL byte, then the reply.".into())
    })?;
    let ranges = highlight_changes(original, reply)
        .map(|diff| diff.changed_ranges())
        .unwrap_or_default();
    let lines: String = ranges
        .iter()
        .map(|(start, end)| format!("{start} {end}\n"))
        .collect();
    output::write(&lines)
}
