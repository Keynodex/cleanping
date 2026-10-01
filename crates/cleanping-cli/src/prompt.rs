//! `cleanping prompt ...`: the system prompt sent with every rewrite.

use cleanping_core::domain::errors::{CleanpingError, Result};
use cleanping_core::domain::prompt_presets::{
    is_earlier_text, preset_for_body, preset_named, PROMPT_PRESETS,
};

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
        PromptAction::Presets => list_presets(services),
        PromptAction::Use { name, yes } => use_preset(services, name, *yes),
    }
}

fn list_presets(services: &Services) -> Result<()> {
    let saved = services.prompts.current()?;
    let current = preset_for_body(&saved);
    for preset in PROMPT_PRESETS {
        let marker = if current == Some(preset) { "*" } else { " " };
        output::line(&format!(
            "{marker} {:<9} {}",
            preset.name, preset.description
        ))?;
    }
    match current {
        Some(preset) if is_earlier_text(&saved) => output::line(&format!(
            "\nYou have an earlier version of {0}. Get the current one with: cleanping prompt use {0}",
            preset.name
        )),
        _ => Ok(()),
    }
}

fn use_preset(services: &Services, name: &str, yes: bool) -> Result<()> {
    let preset = preset_named(name).ok_or_else(|| {
        let names: Vec<&str> = PROMPT_PRESETS.iter().map(|p| p.name).collect();
        CleanpingError::Validation(format!(
            "Unknown preset. Choose one of: {}.",
            names.join(", ")
        ))
    })?;
    let writing_your_own = preset_for_body(&services.prompts.current()?).is_none();
    if writing_your_own && !yes {
        return Err(CleanpingError::Validation(
            "That would replace the system prompt you wrote yourself (see it with: cleanping \
             prompt show). Add --yes to replace it."
                .into(),
        ));
    }
    services.prompts.save(preset.body)?;
    output::line(&format!(
        "System prompt set to the \u{201c}{}\u{201d} preset.",
        preset.name
    ))
}
