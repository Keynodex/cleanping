//! The system prompt sent with every request.

use super::ports::PromptRepository;
use crate::domain::errors::{CleanpingError, Result};

pub const DEFAULT_INSTRUCTIONS: &str =
    "You are a precise copy editor for a software developer's terminal prompts. \
Fix spelling, grammar, and clarity while retaining the author's intent, tone, \
technical details, and all constraints. Preserve commands, code, flags, file \
paths, identifiers, names, URLs, and error messages exactly. Do not execute \
or answer the request. Do not add facts, requirements, or explanations. \
Return only the edited text, with no quotes or Markdown fences. \
If editing would change technical meaning, leave that portion unchanged.";

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
