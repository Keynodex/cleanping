//! `--marks`: which parts of a reply changed, for the shell key to highlight. Pure text in,
//! text out: no database, no files, no network, so it is safe to run on every key press.

use cleanping_core::domain::diff::highlight_changes;
use cleanping_core::domain::errors::{CleanpingError, Result};

use crate::input;
use crate::output;

/// Stdin holds the original text, a NUL byte, then the reply. Prints the reply's length in
/// characters, then one `start end` line per changed range of the reply, in characters. The
/// length lets a shell that counts bytes (a non-UTF-8 locale) notice that the positions would
/// not line up, and highlight nothing rather than the wrong text. Prints nothing at all when
/// the texts are too large to compare quickly.
pub fn run() -> Result<()> {
    let both = input::read_stdin()?;
    let (original, reply) = both.split_once('\0').ok_or_else(|| {
        CleanpingError::Validation("Expected the original text, a NUL byte, then the reply.".into())
    })?;
    let Some(diff) = highlight_changes(original, reply) else {
        return Ok(());
    };
    let mut lines = format!("{}\n", reply.chars().count());
    for (start, end) in diff.changed_ranges() {
        lines.push_str(&format!("{start} {end}\n"));
    }
    output::write(&lines)
}
