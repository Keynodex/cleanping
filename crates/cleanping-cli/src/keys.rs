//! `cleanping keys ...`: save, list, remove and choose API keys.

use cleanping_core::application::ports::SecretStore;
use cleanping_core::domain::errors::{CleanpingError, Result};
use cleanping_core::domain::models::CredentialInput;
use cleanping_core::domain::providers::{ProviderPreset, PROVIDER_PRESETS};

use crate::args::{AddArgs, KeysAction};
use crate::connection;
use crate::input;
use crate::output;
use crate::services::Services;

fn invalid(message: &str) -> CleanpingError {
    CleanpingError::Validation(message.to_string())
}

fn preset(label: &str) -> Result<&'static ProviderPreset> {
    PROVIDER_PRESETS
        .iter()
        .find(|p| p.label.eq_ignore_ascii_case(label) && !p.api_url.is_empty())
        .ok_or_else(|| invalid("Unknown provider. Use DeepSeek, OpenAI, OpenRouter or \"Ollama (local)\", or give --url."))
}

/// The key: a hidden prompt on a terminal (even with `--key-stdin`, which would echo it),
/// else one line from a pipe when asked to, else nothing (a local model needs none).
fn read_key(from_stdin: bool) -> Result<String> {
    if input::stdin_is_a_terminal() {
        return rpassword::prompt_password("API key (hidden; Enter to skip for a local model): ")
            .map(|key| key.trim().to_string())
            .map_err(|_| invalid("Could not read the key."));
    }
    if from_stdin {
        return input::read_key_line();
    }
    Ok(String::new())
}

fn build_input(args: &AddArgs, api_key: String) -> Result<CredentialInput> {
    let chosen = args.provider.as_deref().map(preset).transpose()?;
    if chosen.is_none() && args.url.is_none() {
        return Err(invalid("Give --provider or --url."));
    }
    let name = args
        .name
        .clone()
        .or_else(|| chosen.map(|p| p.label.to_string()));
    Ok(CredentialInput {
        name: name.ok_or_else(|| invalid("Give --name."))?,
        api_url: args
            .url
            .clone()
            .or_else(|| chosen.map(|p| p.api_url.to_string()))
            .unwrap_or_default(),
        model: args
            .model
            .clone()
            .or_else(|| chosen.map(|p| p.default_model.to_string()))
            .unwrap_or_default(),
        api_key,
    })
}

fn add(services: &Services, args: &AddArgs) -> Result<()> {
    let api_key = read_key(args.key_stdin)?;
    let saved = services.credentials.save(build_input(args, api_key)?)?;
    if services.state.selected_credential_id()?.is_none() {
        services.state.select_credential(saved.id)?;
    }
    output::line(&format!("Saved \u{201c}{}\u{201d}.", saved.name))
}

fn list(services: &Services) -> Result<()> {
    let selected = services.state.selected_credential_id()?;
    for c in services.credentials.list()? {
        let has_key = services
            .secrets()
            .get(&c.name)?
            .is_some_and(|k| !k.is_empty());
        let marker = if c.id == selected { "*" } else { " " };
        output::line(&format!(
            "{marker} {}  {}  {}  [{}]",
            c.name,
            c.model,
            c.api_url,
            if has_key { "key saved" } else { "no key" }
        ))?;
    }
    Ok(())
}

pub fn run(services: &Services, action: &KeysAction) -> Result<()> {
    match action {
        KeysAction::List => list(services),
        KeysAction::Add(args) => add(services, args),
        KeysAction::Remove { name } => {
            let credential = services.credentials.find_by_name(name)?;
            let id = credential
                .id
                .ok_or_else(|| invalid("That key has no id."))?;
            services.credentials.delete(id)?;
            if services.state.selected_credential_id()? == Some(id) {
                services.state.select_credential(None)?;
            }
            output::line(&format!("Removed \u{201c}{}\u{201d}.", credential.name))
        }
        KeysAction::Test { name } => connection::run(services, name.as_deref()),
        KeysAction::Use { name } => {
            let credential = services.credentials.find_by_name(name)?;
            services.state.select_credential(credential.id)?;
            output::line(&format!("Now using \u{201c}{}\u{201d}.", credential.name))
        }
    }
}
