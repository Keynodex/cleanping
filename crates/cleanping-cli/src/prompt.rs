//! `cleanping prompt ...`: the system prompt sent with every rewrite.

use cleanping_core::domain::errors::{CleanpingError, Result};

use crate::args::PromptAction;
use crate::input;
use crate::output;
use crate::services::Services;

pub fn run(services: &Services, action: &PromptAction) -> Result<()> {
    match action {
        PromptAction::Show => output::line(&services.prompts.current()?),
        PromptAction::Set { text } => {
            let body = match text {
                Some(text) => text.clone(),
                None if input::stdin_is_a_terminal() => {
                    return Err(CleanpingError::Validation(
                        "Give the new prompt as text, or pipe it in.".into(),
                    ));
                }
                None => input::read_stdin()?,
            };
            services.prompts.save(&body)?;
            output::line("System prompt saved.")
        }
    }
}
