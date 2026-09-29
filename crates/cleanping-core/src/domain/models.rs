//! Domain entities. Plain data only.

use std::fmt;

/// A saved provider endpoint. The secret lives elsewhere, keyed by `name`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Credential {
    /// Row id assigned by the credential store when it is saved.
    pub id: Option<i64>,
    /// Unique name the user gave this key. The secret store keeps the API key under this name.
    pub name: String,
    /// Chat-completions endpoint the text is sent to. It must pass
    /// [`validate_api_url`](crate::domain::validation::validate_api_url): HTTPS, or plain HTTP
    /// only to `localhost`, `127.0.0.1` or `[::1]`.
    pub api_url: String,
    /// Model name sent with every request to this endpoint.
    pub model: String,
}

/// What the user typed when saving a key. `Debug` redacts the key on purpose.
#[derive(Clone, PartialEq, Eq)]
pub struct CredentialInput {
    /// Name for the key: required, one plain line (see
    /// [`validate_credential_fields`](crate::domain::validation::validate_credential_fields)).
    pub name: String,
    /// Endpoint to send text to; the same rules as [`Credential::api_url`].
    pub api_url: String,
    /// Model to ask the endpoint for; required.
    pub model: String,
    /// The API key as typed. Blank means keep the key already saved under this name.
    pub api_key: String,
}

impl fmt::Debug for CredentialInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CredentialInput")
            .field("name", &self.name)
            .field("api_url", &self.api_url)
            .field("model", &self.model)
            .field("api_key", &"<redacted>")
            .finish()
    }
}

/// Whether a recorded rewrite produced an edit. Stored as the text `ok` or `error`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunStatus {
    /// The provider returned an edit and it was kept.
    Ok,
    /// No edit was kept; the run's `error_message` says why.
    Error,
}

impl RunStatus {
    /// The stored text, `ok` or `error`.
    pub fn as_str(self) -> &'static str {
        match self {
            RunStatus::Ok => "ok",
            RunStatus::Error => "error",
        }
    }

    /// The status for its stored text (exact match); `None` for anything else.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "ok" => Some(RunStatus::Ok),
            "error" => Some(RunStatus::Error),
            _ => None,
        }
    }
}

/// One rewrite attempt, input and output kept together for the history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    /// The text that was sent to be edited.
    pub input_text: String,
    /// The edited text; `None` when no edit was kept.
    pub output_text: Option<String>,
    /// Whether an edit was kept.
    pub status: RunStatus,
    /// Why no edit was kept, as a message for the user; `None` on success.
    pub error_message: Option<String>,
    /// How long the provider call took, in milliseconds.
    pub duration_ms: i64,
    /// Name of the key used. Kept even after that key is deleted.
    pub credential_name: Option<String>,
    /// Model the text was sent to.
    pub model: String,
    /// The system prompt sent with the text.
    pub prompt_text: String,
    /// Row id; `None` until the run is stored.
    pub id: Option<i64>,
    /// When the run was stored, as a UTC stamp such as `2026-09-28T18:30:00+00:00`; `None` until
    /// then.
    pub created_at: Option<String>,
    /// Row id of the key used; `None` once that key is deleted.
    pub credential_id: Option<i64>,
}
