//! Stops text that looks like it holds a secret from being sent to a provider.

use cleanping_core::domain::errors::{CleanpingError, Result};
use cleanping_core::domain::secret_scan::find_secret;

/// The refusal for text that looks like it holds `kind` of secret. It names the kind, never
/// the secret.
pub fn refusal(kind: &str) -> CleanpingError {
    CleanpingError::Validation(format!(
        "The text looks like it contains {kind}, so it was not sent."
    ))
}

pub fn refuse_if_secret(text: &str) -> Result<()> {
    match find_secret(text) {
        Some(kind) => Err(refusal(kind)),
        None => Ok(()),
    }
}
