//! What we can tell about a local model server (Ollama) and what to suggest when a test fails.

use url::Url;

use super::validation::is_local_url;

/// What answered at the address of a local provider.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LocalServer {
    /// Nothing answered, and no `ollama` program was found.
    NotInstalled,
    /// Nothing answered, but the `ollama` program is installed.
    NotRunning,
    /// Ollama answered; these are the models it has.
    Running {
        /// Model names exactly as Ollama lists them, such as `qwen2.5:7b` or `llama3:latest`.
        models: Vec<String>,
    },
    /// Something answered, but not like Ollama.
    Other,
}

/// What the user can do about it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalFix {
    /// Install Ollama (nothing answered and the `ollama` program was not found).
    Install,
    /// Start Ollama (the program is installed but nothing answered).
    Start,
    /// Download the configured model (Ollama is running but does not have it).
    PullModel,
}

/// The port Ollama listens on by default.
pub const OLLAMA_PORT: u16 = 11434;

/// True when `wanted` is one of `models`. A name without a tag also matches its `:latest`
/// entry: `llama3` matches `llama3:latest`, not `llama3:8b`.
pub fn model_present(models: &[String], wanted: &str) -> bool {
    let with_default_tag = format!("{wanted}:latest");
    models
        .iter()
        .any(|name| name == wanted || (!wanted.contains(':') && *name == with_default_tag))
}

/// True for a loopback address on Ollama's standard port.
pub fn is_default_ollama_url(api_url: &str) -> bool {
    is_local_url(api_url)
        && Url::parse(api_url.trim()).is_ok_and(|url| url.port() == Some(OLLAMA_PORT))
}

/// What to suggest, if anything. `default_address` says the provider's URL is the standard
/// Ollama address: only then is "not installed" or "not running" advice sensible.
pub fn diagnose(server: &LocalServer, model: &str, default_address: bool) -> Option<LocalFix> {
    match server {
        LocalServer::Running { models } if !model_present(models, model) => {
            Some(LocalFix::PullModel)
        }
        LocalServer::NotInstalled if default_address => Some(LocalFix::Install),
        LocalServer::NotRunning if default_address => Some(LocalFix::Start),
        _ => None,
    }
}

#[cfg(test)]
#[path = "local_server_tests.rs"]
mod tests;
