//! Warnings on stderr when a reply changed a command in the text (a closed quote, a dropped
//! flag, a changed path). The reply is still used; the warning says what to check.

use cleanping_core::domain::command_change::command_changes;

use crate::output;

/// More than this many changes are counted instead of listed.
const LISTED: usize = 5;

/// Say on stderr, one line each, how the reply changed the commands in `original`.
pub fn warn_about(original: &str, reply: &str) {
    let changes = command_changes(original, reply);
    for change in changes.iter().take(LISTED) {
        output::warn(&format!("warning: {change}."));
    }
    if changes.len() > LISTED {
        output::warn(&format!(
            "warning: and {} more changes to commands.",
            changes.len() - LISTED
        ));
    }
}
