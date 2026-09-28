//! Domain entities. Plain data only.

use std::fmt;

/// A saved provider endpoint. The secret lives elsewhere, keyed by `name`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Credential {
    pub id: Option<i64>,
    pub name: String,
    pub api_url: String,
    pub model: String,
}

/// What the user typed when saving a key. `Debug` redacts the key on purpose.
#[derive(Clone, PartialEq, Eq)]
pub struct CredentialInput {
    pub name: String,
    pub api_url: String,
    pub model: String,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunStatus {
    Ok,
    Error,
}

impl RunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            RunStatus::Ok => "ok",
            RunStatus::Error => "error",
        }
    }

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
    pub input_text: String,
    pub output_text: Option<String>,
    pub status: RunStatus,
    pub error_message: Option<String>,
    pub duration_ms: i64,
    pub credential_name: Option<String>,
    pub model: String,
    pub prompt_text: String,
    pub id: Option<i64>,
    pub created_at: Option<String>,
    pub credential_id: Option<i64>,
}
