//! Seams between use cases and infrastructure. Implementations live in `infrastructure`.

use crate::domain::errors::Result;
use crate::domain::models::{Credential, CredentialInput, Run};

/// Everything one rewrite call needs. No `Debug` on purpose: it carries the API key.
pub struct RewriteRequest<'a> {
    pub text: &'a str,
    pub instructions: &'a str,
    pub api_url: &'a str,
    pub api_key: &'a str,
    pub model: &'a str,
}

pub trait Rewriter {
    /// The rewritten text: trimmed and free of terminal control characters.
    fn rewrite(&self, request: &RewriteRequest<'_>) -> Result<String>;
}

pub trait CredentialRepository {
    fn list(&self) -> Result<Vec<Credential>>;
    fn get(&self, id: i64) -> Result<Credential>;
    fn upsert(&self, draft: &CredentialInput) -> Result<Credential>;
    fn delete(&self, id: i64) -> Result<()>;
}

pub trait SecretStore {
    fn get(&self, name: &str) -> Result<Option<String>>;
    fn set(&self, name: &str, secret: &str) -> Result<()>;
    fn delete(&self, name: &str) -> Result<()>;
}

pub trait RunRepository {
    fn add(&self, run: &Run) -> Result<Run>;
    fn recent(&self, limit: usize) -> Result<Vec<Run>>;
    /// Remove every run; returns how many were removed.
    fn delete_all(&self) -> Result<usize>;
    /// Remove runs stamped before `cutoff` (same UTC format as `created_at`); returns the count.
    fn delete_before(&self, cutoff: &str) -> Result<usize>;
}

pub trait PromptRepository {
    fn current(&self) -> Result<Option<String>>;
    fn save(&self, body: &str) -> Result<()>;
}

pub trait StateRepository {
    fn get(&self, key: &str) -> Result<Option<String>>;
    fn set(&self, key: &str, value: &str) -> Result<()>;
}
