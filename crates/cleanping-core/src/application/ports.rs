//! Seams between use cases and infrastructure. Implementations live in `infrastructure`.

use crate::domain::errors::Result;
use crate::domain::models::{Credential, CredentialInput, Run};

/// Everything one rewrite call needs. No `Debug` on purpose: it carries the API key.
pub struct RewriteRequest<'a> {
    /// The text to edit.
    pub text: &'a str,
    /// The system prompt: how to edit it.
    pub instructions: &'a str,
    /// The chat-completions endpoint to send it to.
    pub api_url: &'a str,
    /// The API key; empty when none is saved for a local endpoint.
    pub api_key: &'a str,
    /// The model to ask for.
    pub model: &'a str,
}

/// Sends text to the user's provider and returns the edit.
///
/// Implementations must send nothing to a URL that
/// [`validate_api_url`](crate::domain::validation::validate_api_url) rejects, must not follow
/// redirects, and must keep the key and the provider's response body out of error messages.
/// Report a provider failure as `CleanpingError::Rewrite`:
/// [`PolishText`](crate::application::polisher::PolishText) records it as a failed run, while
/// any other error stops the run before anything is recorded.
pub trait Rewriter {
    /// The rewritten text: trimmed and free of terminal control characters.
    fn rewrite(&self, request: &RewriteRequest<'_>) -> Result<String>;
}

/// Saved endpoints (name, URL, model). The API key is never stored here: it lives in a
/// [`SecretStore`] under the credential's name.
pub trait CredentialRepository {
    /// Every saved credential.
    fn list(&self) -> Result<Vec<Credential>>;
    /// The credential with this id; a `NotFound` error if there is none.
    fn get(&self, id: i64) -> Result<Credential>;
    /// Insert `draft`, or update the URL and model of the credential with exactly the same name.
    /// Returns the stored credential with its id; `draft.api_key` is ignored.
    fn upsert(&self, draft: &CredentialInput) -> Result<Credential>;
    /// Remove the credential with this id. An unknown id is not an error.
    fn delete(&self, id: i64) -> Result<()>;
}

/// API keys, looked up by credential name. Implementations must never log or display a value.
pub trait SecretStore {
    /// The key saved under `name`, if any.
    fn get(&self, name: &str) -> Result<Option<String>>;
    /// Save `secret` under `name`, replacing any earlier value.
    fn set(&self, name: &str, secret: &str) -> Result<()>;
    /// Remove the key saved under `name`. A missing name is not an error.
    fn delete(&self, name: &str) -> Result<()>;
}

/// The local rewrite history.
pub trait RunRepository {
    /// Store `run`; returns it with `id` and `created_at` filled in.
    fn add(&self, run: &Run) -> Result<Run>;
    /// At most `limit` runs, newest first.
    fn recent(&self, limit: usize) -> Result<Vec<Run>>;
    /// Remove every run; returns how many were removed.
    fn delete_all(&self) -> Result<usize>;
    /// Remove runs stamped before `cutoff` (same UTC format as `created_at`); returns the count.
    fn delete_before(&self, cutoff: &str) -> Result<usize>;
}

/// The saved system prompt.
pub trait PromptRepository {
    /// The most recently saved prompt, or `None` if none was saved.
    fn current(&self) -> Result<Option<String>>;
    /// Save `body` as the current prompt. Implementations may keep earlier prompts; the SQLite
    /// one does (append-only).
    fn save(&self, body: &str) -> Result<()>;
}

/// Small named values that front-ends restore on the next launch.
pub trait StateRepository {
    /// The value saved under `key`, if any.
    fn get(&self, key: &str) -> Result<Option<String>>;
    /// Save `value` under `key`, replacing any earlier value.
    fn set(&self, key: &str, value: &str) -> Result<()>;
}

/// What the release server said about the newest published release. Both fields are untrusted
/// text from the network: the use case validates them before anything is shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LatestRelease {
    /// The release's tag, for example `v0.5.0`.
    pub tag_name: String,
    /// The release's web page, if the reply named one.
    pub html_url: Option<String>,
}

/// Asks where CleanPing is published for its newest release.
///
/// Implementations must send nothing about the user (no ids, history, keys or text), must not
/// follow redirects, must limit the size of the reply and must keep its body out of error
/// messages. Report every failure as `CleanpingError::UpdateCheck`.
pub trait ReleaseSource {
    /// The newest published release.
    fn latest(&self) -> Result<LatestRelease>;
}
