//! Polish one text and record the attempt in the history.

use std::time::Instant;

use super::ports::{RewriteRequest, Rewriter, RunRepository, SecretStore};
use crate::domain::errors::{CleanpingError, Result};
use crate::domain::models::{Credential, Run, RunStatus};
use crate::domain::shape::keeps_shape;
use crate::domain::validation::is_local_url;

/// What one polish attempt produced. Exactly one of `output_text` and `error_message` is set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolishResult {
    /// The edited text; `None` when the attempt failed.
    pub output_text: Option<String>,
    /// Why the attempt failed, as a message for the user; `None` on success.
    pub error_message: Option<String>,
    /// The attempt as recorded in the history, with its id and time.
    pub run: Run,
}

impl PolishResult {
    /// True when the attempt produced an edit.
    pub fn ok(&self) -> bool {
        self.error_message.is_none()
    }
}

const DOES_NOT_FIT: &str = "The reply is longer or has more lines than your text; not applied.";

/// Runs the provider call, then persists an ok/error row either way.
pub struct PolishText<W: Rewriter, R: RunRepository, S: SecretStore> {
    rewriter: W,
    runs: R,
    secrets: S,
    keep_shape: bool,
}

impl<W: Rewriter, R: RunRepository, S: SecretStore> PolishText<W, R, S> {
    /// Build the use case. Replies are not shape-checked unless
    /// [`keeping_shape`](Self::keeping_shape) is called.
    pub fn new(rewriter: W, runs: R, secrets: S) -> Self {
        Self {
            rewriter,
            runs,
            secrets,
            keep_shape: false,
        }
    }

    /// Refuse a reply that could hide part of itself when it replaces the text on a command
    /// line: more lines, much longer, or padded with blanks (see `keeps_shape`).
    pub fn keeping_shape(mut self) -> Self {
        self.keep_shape = true;
        self
    }

    fn misfits(&self, text: &str, output: &str) -> bool {
        self.keep_shape && !keeps_shape(text, output)
    }

    /// `Err` means the run never happened (missing key, invalid URL, storage failure).
    /// A provider failure is an `Ok` result with `error_message` set and a recorded run.
    pub fn run(
        &self,
        text: &str,
        instructions: &str,
        credential: &Credential,
    ) -> Result<PolishResult> {
        let api_key = self.secrets.get(&credential.name)?.unwrap_or_default();
        if api_key.is_empty() && !is_local_url(&credential.api_url) {
            return Err(CleanpingError::MissingCredential(format!(
                "No API key saved for \u{201c}{}\u{201d}.",
                credential.name
            )));
        }
        let started = Instant::now();
        let outcome = self.rewriter.rewrite(&RewriteRequest {
            text,
            instructions,
            api_url: &credential.api_url,
            api_key: &api_key,
            model: &credential.model,
        });
        let (output_text, error_message) = match outcome {
            Ok(output) if self.misfits(text, &output) => (None, Some(DOES_NOT_FIT.to_string())),
            Ok(output) => (Some(output), None),
            Err(CleanpingError::Rewrite(message)) => (None, Some(message)),
            Err(other) => return Err(other),
        };
        let run = self.runs.add(&Run {
            input_text: text.to_string(),
            output_text: output_text.clone(),
            status: if error_message.is_some() {
                RunStatus::Error
            } else {
                RunStatus::Ok
            },
            error_message: error_message.clone(),
            duration_ms: i64::try_from(started.elapsed().as_millis()).unwrap_or(i64::MAX),
            credential_name: Some(credential.name.clone()),
            model: credential.model.clone(),
            prompt_text: instructions.to_string(),
            id: None,
            created_at: None,
            credential_id: credential.id,
        })?;
        Ok(PolishResult {
            output_text,
            error_message,
            run,
        })
    }
}

#[cfg(test)]
#[path = "polisher_tests.rs"]
mod tests;
