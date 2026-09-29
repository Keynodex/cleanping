//! Step 1: choose the AI, save its key, and, for a local model, check that it is ready.

use cleanping_core::application::ports::SecretStore;
use cleanping_core::domain::errors::{CleanpingError, Result};
use cleanping_core::domain::models::{Credential, CredentialInput};
use cleanping_core::domain::providers::PROVIDER_PRESETS;
use cleanping_core::domain::validation::{is_local_url, validate_credential_fields};

use super::ask::{choose, confirm};
use super::console::{too_many_bad_answers, Console};
use super::local_model;
use super::tools::Tools;
use crate::services::Services;

/// Tries at typing a custom provider, or at getting a key saved, before giving up.
const TRIES: usize = 3;
/// Blank keys in a row before giving up.
const KEY_TRIES: usize = 5;

/// A provider's name, address and model, already checked.
struct Draft {
    name: String,
    api_url: String,
    model: String,
}

/// Ask which AI to use, save it (with its key, typed hidden) and make it the one in use.
pub fn run(
    console: &mut dyn Console,
    tools: &dyn Tools,
    services: &Services,
) -> Result<Credential> {
    let draft = pick(console)?;
    let saved = save(console, services, &draft)?;
    services.state.select_credential(saved.id)?;
    console.say(&format!("Saved. Now using \u{201c}{}\u{201d}.", saved.name));
    local_model::prepare(console, tools, &saved)?;
    Ok(saved)
}

fn option_text(label: &str) -> String {
    match label {
        "Ollama (local)" => format!("{label} - runs on this computer; nothing leaves it"),
        "Custom" => format!("{label} - any OpenAI-compatible address"),
        other => other.to_string(),
    }
}

fn pick(console: &mut dyn Console) -> Result<Draft> {
    let options: Vec<String> = PROVIDER_PRESETS
        .iter()
        .map(|p| option_text(p.label))
        .collect();
    let index = choose(console, "Step 1: choose your AI", &options, None)?;
    let preset = &PROVIDER_PRESETS[index];
    if preset.api_url.is_empty() {
        return custom(console);
    }
    Ok(Draft {
        name: preset.label.to_string(),
        api_url: preset.api_url.to_string(),
        model: preset.default_model.to_string(),
    })
}

fn custom(console: &mut dyn Console) -> Result<Draft> {
    for _ in 0..TRIES {
        let name = console.ask("Name for this key (for example My server): ")?;
        let api_url = console.ask(
            "API address (https://..., or http://localhost... for a server on this computer): ",
        )?;
        let model = console.ask("Model name: ")?;
        let checked = validate_credential_fields(CredentialInput {
            name,
            api_url,
            model,
            api_key: String::new(),
        });
        match checked {
            Ok(fields) => {
                return Ok(Draft {
                    name: fields.name,
                    api_url: fields.api_url,
                    model: fields.model,
                })
            }
            Err(CleanpingError::Validation(message)) => console.say(&message),
            Err(error) => return Err(error),
        }
    }
    Err(too_many_bad_answers())
}

fn ask_key(console: &mut dyn Console) -> Result<String> {
    for _ in 0..KEY_TRIES {
        let key = console.ask_secret("API key (typing is hidden): ")?;
        if !key.trim().is_empty() {
            return Ok(key.trim().to_string());
        }
        console.say("A key is needed for this provider.");
    }
    Err(too_many_bad_answers())
}

fn save(console: &mut dyn Console, services: &Services, draft: &Draft) -> Result<Credential> {
    let local = is_local_url(&draft.api_url);
    let has_saved_key = !local
        && services
            .secrets()
            .get(&draft.name)?
            .is_some_and(|key| !key.is_empty());
    let keep = has_saved_key
        && confirm(
            console,
            &format!(
                "A key for \u{201c}{}\u{201d} is already saved. Keep it?",
                draft.name
            ),
            true,
        )?;
    let mut key = if local || keep {
        String::new()
    } else {
        ask_key(console)?
    };
    for attempt in 0..TRIES {
        let saved = services.credentials.save(CredentialInput {
            name: draft.name.clone(),
            api_url: draft.api_url.clone(),
            model: draft.model.clone(),
            api_key: key.clone(),
        });
        match saved {
            Ok(saved) => {
                if !key.is_empty() {
                    console.say("The key is saved in a file only you can read.");
                }
                return Ok(saved);
            }
            Err(CleanpingError::Validation(message)) if attempt + 1 < TRIES => {
                console.say(&message);
                key = if local {
                    String::new()
                } else {
                    ask_key(console)?
                };
            }
            Err(error) => return Err(error),
        }
    }
    Err(too_many_bad_answers())
}

#[cfg(test)]
#[path = "provider_tests.rs"]
mod tests;
