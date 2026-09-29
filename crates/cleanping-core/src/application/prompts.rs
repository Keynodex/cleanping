//! The system prompt sent with every request.

use super::ports::PromptRepository;
use crate::domain::errors::{CleanpingError, Result};

pub use crate::domain::prompt_presets::DEFAULT_INSTRUCTIONS;

pub struct PromptService<P: PromptRepository> {
    prompts: P,
}

impl<P: PromptRepository> PromptService<P> {
    pub fn new(prompts: P) -> Self {
        Self { prompts }
    }

    pub fn current(&self) -> Result<String> {
        let saved = self.prompts.current()?.filter(|body| !body.is_empty());
        Ok(saved.unwrap_or_else(|| DEFAULT_INSTRUCTIONS.to_string()))
    }

    pub fn save(&self, body: &str) -> Result<()> {
        let text = body.trim();
        if text.is_empty() {
            return Err(CleanpingError::Validation(
                "System prompt must not be empty.".into(),
            ));
        }
        self.prompts.save(text)
    }
}

#[cfg(test)]
#[path = "prompts_tests.rs"]
mod tests;
