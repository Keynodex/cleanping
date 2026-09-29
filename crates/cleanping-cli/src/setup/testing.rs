//! Test-only stand-ins for the wizard's outside world.

use std::collections::VecDeque;

use cleanping_core::domain::errors::Result;

use super::console::{stopped, Console};

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
