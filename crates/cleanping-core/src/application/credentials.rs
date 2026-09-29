//! Save, list and delete provider credentials.

use super::ports::{CredentialRepository, SecretStore};
use crate::domain::errors::{CleanpingError, Result};
use crate::domain::models::{Credential, CredentialInput};
use crate::domain::validation::{is_local_url, same_origin, validate_credential_fields};

/// Keeps the stored row (name/url/model) and the secret store in sync.
pub struct CredentialService<R: CredentialRepository, S: SecretStore> {
    credentials: R,
    secrets: S,
}

impl<R: CredentialRepository, S: SecretStore> CredentialService<R, S> {
    /// Build the service on the credential rows and the secret store.
    pub fn new(credentials: R, secrets: S) -> Self {
        Self {
            credentials,
            secrets,
        }
    }

    /// Every saved credential.
    pub fn list(&self) -> Result<Vec<Credential>> {
        self.credentials.list()
    }

    /// The credential with this id; a `NotFound` error if it is gone.
    pub fn get(&self, id: i64) -> Result<Credential> {
        self.credentials.get(id)
    }

    /// A key by name: the exact spelling wins; otherwise a unique case-insensitive match.
    pub fn find_by_name(&self, name: &str) -> Result<Credential> {
        let all = self.credentials.list()?;
        if let Some(exact) = all.iter().find(|c| c.name == name) {
            return Ok(exact.clone());
        }
        let mut similar = all.iter().filter(|c| c.name.eq_ignore_ascii_case(name));
        match (similar.next(), similar.next()) {
            (Some(only), None) => Ok(only.clone()),
            (Some(_), Some(_)) => Err(CleanpingError::Validation(format!(
                "Several keys match \u{201c}{name}\u{201d}; use the exact name."
            ))),
            _ => Err(CleanpingError::NotFound(format!(
                "No saved key named \u{201c}{name}\u{201d}."
            ))),
        }
    }

    /// Which key to use: the named one, else the selected one, else the only one.
    /// Never guesses between several keys, so text cannot go to a provider nobody chose.
    pub fn resolve(&self, name: Option<&str>, selected: Option<i64>) -> Result<Credential> {
        if let Some(name) = name {
            return self.find_by_name(name);
        }
        let mut all = self.credentials.list()?;
        if all.is_empty() {
            return Err(CleanpingError::MissingCredential(
                "No API key saved yet.".into(),
            ));
        }
        if let Some(index) = all.iter().position(|c| c.id.is_some() && c.id == selected) {
            return Ok(all.swap_remove(index));
        }
        if all.len() == 1 {
            return Ok(all.swap_remove(0));
        }
        Err(CleanpingError::MissingCredential(
            "Several keys are saved and none is selected.".into(),
        ))
    }

    /// A blank key means "keep the stored one"; only local URLs may stay keyless. The row is
    /// written before the secret and rolled back if the secret cannot be stored, so a failure
    /// never leaves a half-saved key behind.
    pub fn save(&self, draft: CredentialInput) -> Result<Credential> {
        let normalized = validate_credential_fields(draft)?;
        let existing = self.credentials.list()?;
        let previous = existing.iter().find(|c| c.name == normalized.name);
        // A new name may not be a case variant of another key. An exact match is an update and
        // is always allowed, even when older versions left two names that differ by case.
        if previous.is_none()
            && existing
                .iter()
                .any(|c| c.name.eq_ignore_ascii_case(&normalized.name))
        {
            return Err(CleanpingError::Validation(format!(
                "Another key\u{2019}s name differs from \u{201c}{}\u{201d} only by case; use that exact name.",
                normalized.name
            )));
        }
        if normalized.api_key.is_empty() {
            self.check_blank_key_is_allowed(&normalized, previous)?;
        }
        let saved = self.credentials.upsert(&normalized)?;
        if !normalized.api_key.is_empty() {
            if let Err(error) = self.secrets.set(&saved.name, &normalized.api_key) {
                self.undo_upsert(&saved, previous);
                return Err(error);
            }
        }
        Ok(saved)
    }

    fn check_blank_key_is_allowed(
        &self,
        draft: &CredentialInput,
        previous: Option<&Credential>,
    ) -> Result<()> {
        let stored = self.secrets.get(&draft.name)?.filter(|s| !s.is_empty());
        if stored.is_some() && previous.is_some_and(|p| !same_origin(&p.api_url, &draft.api_url)) {
            return Err(CleanpingError::Validation(
                "The API address changed; enter the API key again so the saved key is not sent to a new host."
                    .into(),
            ));
        }
        // A secret with no row (say the database was deleted) has no known address to compare,
        // so it is never adopted for a new one.
        if stored.is_some() && previous.is_none() {
            return Err(CleanpingError::Validation(format!(
                "A key for \u{201c}{}\u{201d} is left over from an earlier setup; enter the API key again.",
                draft.name
            )));
        }
        if stored.is_none() && !is_local_url(&draft.api_url) {
            return Err(CleanpingError::Validation("Enter an API key.".into()));
        }
        Ok(())
    }

    /// Best effort: put the row back the way it was after a failed secret write.
    fn undo_upsert(&self, saved: &Credential, previous: Option<&Credential>) {
        match (previous, saved.id) {
            (None, Some(id)) => {
                let _ = self.credentials.delete(id);
            }
            (Some(old), _) => {
                let _ = self.credentials.upsert(&CredentialInput {
                    name: old.name.clone(),
                    api_url: old.api_url.clone(),
                    model: old.model.clone(),
                    api_key: String::new(),
                });
            }
            _ => {}
        }
    }

    /// The secret goes first: if it cannot be removed the row stays and the delete can be retried.
    pub fn delete(&self, id: i64) -> Result<()> {
        let credential = self.credentials.get(id)?;
        self.secrets.delete(&credential.name)?;
        self.credentials.delete(id)
    }
}

#[cfg(test)]
#[path = "credentials_tests.rs"]
mod tests;
