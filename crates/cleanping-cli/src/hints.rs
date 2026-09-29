//! Copy-pasteable commands shown inside error messages.

use cleanping_core::domain::errors::CleanpingError;
use cleanping_core::domain::models::Credential;

pub const ADD_FIRST_KEY: &str = "Add one: cleanping keys add --provider OpenAI";
pub const CHOOSE_A_KEY: &str = "Choose one: cleanping keys use NAME (or pass -c NAME).";
pub const SEE_KEYS: &str = "See: cleanping keys list";

/// Single-quoted for POSIX shells, so a name or URL cannot inject anything when pasted.
pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// The command that re-saves this key. On a terminal it asks for the key with a hidden prompt.
pub fn add_saved_key(credential: &Credential) -> String {
    format!(
        "cleanping keys add --name {} --url {} --model {}",
        shell_quote(&credential.name),
        shell_quote(&credential.api_url),
        shell_quote(&credential.model)
    )
}

/// A missing-key error, plus the command that saves this key again.
pub fn with_add_hint(error: CleanpingError, credential: &Credential) -> CleanpingError {
    match error {
        CleanpingError::MissingCredential(message) => CleanpingError::MissingCredential(format!(
            "{message} Add it: {}",
            add_saved_key(credential)
        )),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoting_survives_quotes_spaces_and_dollars() {
        assert_eq!(shell_quote("plain"), "'plain'");
        assert_eq!(shell_quote("it's $HOME"), "'it'\\''s $HOME'");
    }
}
