//! Test-only stand-ins for the wizard's outside world.

use std::cell::RefCell;
use std::collections::VecDeque;

use cleanping_core::application::connection_check::CheckOutcome;
use cleanping_core::domain::errors::{CleanpingError, Result};
use cleanping_core::domain::local_server::LocalServer;
use cleanping_core::domain::models::Credential;

use super::console::{stopped, Console};
use super::tools::Tools;
use crate::services::Services;

/// Answers from a script. Secrets are kept apart and never appear in `said`.
#[derive(Default)]
pub struct ScriptedConsole {
    answers: VecDeque<String>,
    secrets: VecDeque<String>,
    pub said: Vec<String>,
    pub questions: Vec<String>,
}

impl ScriptedConsole {
    pub fn new(answers: &[&str]) -> Self {
        Self {
            answers: answers.iter().map(|a| (*a).to_string()).collect(),
            ..Self::default()
        }
    }

    pub fn with_secrets(mut self, secrets: &[&str]) -> Self {
        self.secrets = secrets.iter().map(|s| (*s).to_string()).collect();
        self
    }

    pub fn said_text(&self) -> String {
        self.said.join("\n")
    }

    /// Answers and secrets not used yet.
    pub fn left_over(&self) -> usize {
        self.answers.len() + self.secrets.len()
    }
}

impl Console for ScriptedConsole {
    fn say(&mut self, text: &str) {
        self.said.push(text.to_string());
    }

    fn ask(&mut self, question: &str) -> Result<String> {
        self.questions.push(question.to_string());
        self.answers.pop_front().ok_or_else(stopped)
    }

    fn ask_secret(&mut self, question: &str) -> Result<String> {
        self.questions.push(question.to_string());
        self.secrets.pop_front().ok_or_else(stopped)
    }
}

/// Services on a temp folder: real database and key file, nothing of the user's.
pub fn temp_services() -> (tempfile::TempDir, Services) {
    let dir = tempfile::tempdir().unwrap();
    let services = Services::open_at(
        dir.path().join("cleanping.db"),
        dir.path().join("secrets.json"),
    )
    .unwrap();
    (dir, services)
}

/// The outside world, decided by the test.
pub struct FakeTools {
    pub server: Option<LocalServer>,
    pub pull_works: bool,
    pub pulled: RefCell<Vec<String>>,
    /// `None`: the test passes. `Some(message)`: it fails with that message.
    pub test_failure: Option<String>,
    pub tested: RefCell<Vec<String>>,
}

impl Default for FakeTools {
    fn default() -> Self {
        Self {
            server: None,
            pull_works: true,
            pulled: RefCell::default(),
            test_failure: None,
            tested: RefCell::default(),
        }
    }
}

impl Tools for FakeTools {
    fn probe(&self, _api_url: &str) -> Option<LocalServer> {
        self.server.clone()
    }

    fn pull(&self, model: &str) -> bool {
        self.pulled.borrow_mut().push(model.to_string());
        self.pull_works
    }

    fn test(&self, _services: &Services, credential: &Credential) -> Result<CheckOutcome> {
        self.tested.borrow_mut().push(credential.name.clone());
        match &self.test_failure {
            None => Ok(CheckOutcome { duration_ms: 42 }),
            Some(message) => Err(CleanpingError::Rewrite(message.clone())),
        }
    }
}
