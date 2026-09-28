//! What the front-ends restore on the next launch.

use super::ports::StateRepository;
use crate::domain::errors::Result;
use crate::domain::versions::VersionStack;

const SELECTED_CREDENTIAL: &str = "selected_credential_id";
const VERSIONS: &str = "versions";
const LEGACY_DRAFT: &str = "draft";
const LEGACY_RESULT: &str = "result";

pub struct AppState<S: StateRepository> {
    state: S,
}

impl<S: StateRepository> AppState<S> {
    pub fn new(state: S) -> Self {
        Self { state }
    }

    pub fn selected_credential_id(&self) -> Result<Option<i64>> {
        Ok(self
            .state
            .get(SELECTED_CREDENTIAL)?
            .and_then(|raw| raw.parse().ok()))
    }

    pub fn select_credential(&self, id: Option<i64>) -> Result<()> {
        self.state.set(
            SELECTED_CREDENTIAL,
            &id.map(|i| i.to_string()).unwrap_or_default(),
        )
    }

    /// The editor history; adopts the old draft/result pair the first time.
    pub fn load_versions(&self) -> Result<VersionStack> {
        if let Some(raw) = self.state.get(VERSIONS)?.filter(|raw| !raw.is_empty()) {
            return Ok(VersionStack::loads(&raw));
        }
        let mut legacy = Vec::new();
        for key in [LEGACY_DRAFT, LEGACY_RESULT] {
            if let Some(text) = self.state.get(key)?.filter(|t| !t.trim().is_empty()) {
                legacy.push(text);
            }
        }
        Ok(VersionStack::from_parts(legacy, None))
    }

    pub fn save_versions(&self, stack: &VersionStack) -> Result<()> {
        self.state.set(VERSIONS, &stack.dumps())?;
        for key in [LEGACY_DRAFT, LEGACY_RESULT] {
            // Adopted into the stack; leave no stale copy of the user's text.
            if self.state.get(key)?.is_some_and(|text| !text.is_empty()) {
                self.state.set(key, "")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "app_state_tests.rs"]
mod tests;
