//! The writer is for people at a keyboard: it needs a terminal for both input and output.

use cleanping_core::domain::errors::{CleanpingError, Result};

/// `Ok` when both stdin and stdout are terminals, otherwise a usage error (exit code 2).
pub fn require(stdin_is_terminal: bool, stdout_is_terminal: bool) -> Result<()> {
    if stdin_is_terminal && stdout_is_terminal {
        return Ok(());
    }
    Err(CleanpingError::Validation(
        "cleanping writer needs a terminal for its input and its output. Run it in a \
         terminal window, not in a script or a pipe."
            .into(),
    ))
}

#[cfg(test)]
#[path = "terminal_tests.rs"]
mod tests;
