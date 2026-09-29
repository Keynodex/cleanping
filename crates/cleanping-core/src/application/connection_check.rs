//! Checks that a saved provider answers: one tiny fixed request, nothing of the user's.

use std::time::Instant;

use super::ports::{RewriteRequest, Rewriter, SecretStore};
use crate::domain::errors::{CleanpingError, Result};
use crate::domain::models::Credential;
use crate::domain::validation::is_local_url;

/// Fixed words, so a test can never carry the user's text or history to a provider.
pub const CHECK_INSTRUCTIONS: &str = "This is a connection test. Reply with the single word OK.";
/// The text a connection test asks the provider to edit (with [`CHECK_INSTRUCTIONS`]).
pub const CHECK_TEXT: &str = "ping";

/// A connection test that succeeded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckOutcome {
    /// How long the provider took to answer, in milliseconds.
    pub duration_ms: u64,
}

/// Use case: check that a saved credential works by sending one fixed request. The user's
/// text is never sent and nothing is written to the history.
pub struct ConnectionCheck<W: Rewriter, S: SecretStore> {
    rewriter: W,
    secrets: S,
}

impl<W: Rewriter, S: SecretStore> ConnectionCheck<W, S> {
    /// Build the check from a rewriter and the store that holds the keys.
    pub fn new(rewriter: W, secrets: S) -> Self {
        Self { rewriter, secrets }
    }

    /// `Ok` when the provider answered. Any failure is an `Err` with a message that never
    /// contains the key or the provider's response body.
    pub fn run(&self, credential: &Credential) -> Result<CheckOutcome> {
        let api_key = self.secrets.get(&credential.name)?.unwrap_or_default();
        if api_key.is_empty() && !is_local_url(&credential.api_url) {
            return Err(CleanpingError::MissingCredential(format!(
                "No API key saved for \u{201c}{}\u{201d}.",
                credential.name
            )));
        }
        let started = Instant::now();
        self.rewriter.rewrite(&RewriteRequest {
            text: CHECK_TEXT,
            instructions: CHECK_INSTRUCTIONS,
            api_url: &credential.api_url,
            api_key: &api_key,
            model: &credential.model,
        })?;
        Ok(CheckOutcome {
            duration_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        })
    }
}

#[cfg(test)]
#[path = "connection_check_tests.rs"]
mod tests;
