//! In-memory fakes for the ports. Test-only.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use crate::application::ports::*;
use crate::domain::errors::{CleanpingError, Result};
use crate::domain::models::{Credential, CredentialInput, Run};

#[derive(Default)]
pub struct FakeSecrets {
    pub items: RefCell<HashMap<String, String>>,
    /// When set, every write (`set`, `delete`) fails, like a full disk.
    pub failing: Cell<bool>,
}

impl FakeSecrets {
    pub fn with(name: &str, secret: &str) -> Self {
        let fake = Self::default();
        fake.items.borrow_mut().insert(name.into(), secret.into());
        fake
    }
}

fn disk_full() -> CleanpingError {
    CleanpingError::Storage("The disk is full.".into())
}

impl SecretStore for FakeSecrets {
    fn get(&self, name: &str) -> Result<Option<String>> {
        Ok(self.items.borrow().get(name).cloned())
    }
    fn set(&self, name: &str, secret: &str) -> Result<()> {
        if self.failing.get() {
            return Err(disk_full());
        }
        self.items.borrow_mut().insert(name.into(), secret.into());
        Ok(())
    }
    fn delete(&self, name: &str) -> Result<()> {
        if self.failing.get() {
            return Err(disk_full());
        }
        self.items.borrow_mut().remove(name);
        Ok(())
    }
}

#[derive(Default)]
pub struct FakeCredentials {
    pub items: RefCell<Vec<Credential>>,
    /// When set, every write (`upsert`, `delete`) fails.
    pub failing: Cell<bool>,
}

impl CredentialRepository for FakeCredentials {
    fn list(&self) -> Result<Vec<Credential>> {
        Ok(self.items.borrow().clone())
    }
    fn get(&self, id: i64) -> Result<Credential> {
        self.items
            .borrow()
            .iter()
            .find(|c| c.id == Some(id))
            .cloned()
            .ok_or_else(|| CleanpingError::NotFound(format!("Credential {id} no longer exists.")))
    }
    fn upsert(&self, draft: &CredentialInput) -> Result<Credential> {
        if self.failing.get() {
            return Err(disk_full());
        }
        let mut rows = self.items.borrow_mut();
        let id = rows
            .iter()
            .find(|c| c.name == draft.name)
            .and_then(|c| c.id)
            .unwrap_or_else(|| rows.iter().filter_map(|c| c.id).max().unwrap_or(0) + 1);
        let saved = Credential {
            id: Some(id),
            name: draft.name.clone(),
            api_url: draft.api_url.clone(),
            model: draft.model.clone(),
        };
        rows.retain(|c| c.name != draft.name);
        rows.push(saved.clone());
        Ok(saved)
    }
    fn delete(&self, id: i64) -> Result<()> {
        if self.failing.get() {
            return Err(disk_full());
        }
        self.items.borrow_mut().retain(|c| c.id != Some(id));
        Ok(())
    }
}

#[derive(Default)]
pub struct FakeRuns(pub RefCell<Vec<Run>>);

impl RunRepository for FakeRuns {
    fn add(&self, run: &Run) -> Result<Run> {
        let mut rows = self.0.borrow_mut();
        let stored = Run {
            id: Some(rows.len() as i64 + 1),
            created_at: Some("2026-09-28T00:00:00+00:00".into()),
            ..run.clone()
        };
        rows.push(stored.clone());
        Ok(stored)
    }
    fn recent(&self, limit: usize) -> Result<Vec<Run>> {
        Ok(self.0.borrow().iter().rev().take(limit).cloned().collect())
    }
    fn delete_all(&self) -> Result<usize> {
        Ok(self.0.borrow_mut().drain(..).count())
    }
    fn delete_before(&self, cutoff: &str) -> Result<usize> {
        let mut rows = self.0.borrow_mut();
        let before = rows.len();
        rows.retain(|r| r.created_at.as_deref().is_some_and(|t| t >= cutoff));
        Ok(before - rows.len())
    }
}

#[derive(Default)]
pub struct FakePrompts(pub RefCell<Vec<String>>);

impl PromptRepository for FakePrompts {
    fn current(&self) -> Result<Option<String>> {
        Ok(self.0.borrow().last().cloned())
    }
    fn save(&self, body: &str) -> Result<()> {
        self.0.borrow_mut().push(body.into());
        Ok(())
    }
}

#[derive(Default)]
pub struct FakeState(pub RefCell<HashMap<String, String>>);

impl StateRepository for FakeState {
    fn get(&self, key: &str) -> Result<Option<String>> {
        Ok(self.0.borrow().get(key).cloned())
    }
    fn set(&self, key: &str, value: &str) -> Result<()> {
        self.0.borrow_mut().insert(key.into(), value.into());
        Ok(())
    }
}

/// One recorded call to the fake rewriter.
pub struct RewriteCall {
    pub text: String,
    pub instructions: String,
    pub api_url: String,
    pub api_key: String,
    pub model: String,
}

/// Records every call; returns the configured outcome.
pub struct FakeRewriter {
    pub outcome: Result<String>,
    pub calls: RefCell<Vec<RewriteCall>>,
}

impl FakeRewriter {
    pub fn returning(text: &str) -> Self {
        Self {
            outcome: Ok(text.into()),
            calls: RefCell::default(),
        }
    }
    pub fn failing(error: CleanpingError) -> Self {
        Self {
            outcome: Err(error),
            calls: RefCell::default(),
        }
    }
}

impl Rewriter for FakeRewriter {
    fn rewrite(&self, request: &RewriteRequest<'_>) -> Result<String> {
        self.calls.borrow_mut().push(RewriteCall {
            text: request.text.into(),
            instructions: request.instructions.into(),
            api_url: request.api_url.into(),
            api_key: request.api_key.into(),
            model: request.model.into(),
        });
        match &self.outcome {
            Ok(text) => Ok(text.clone()),
            Err(CleanpingError::Rewrite(m)) => Err(CleanpingError::Rewrite(m.clone())),
            Err(other) => Err(CleanpingError::Storage(other.to_string())),
        }
    }
}
