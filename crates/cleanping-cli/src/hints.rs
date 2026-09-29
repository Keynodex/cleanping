//! Copy-pasteable commands shown inside error messages.

use cleanping_core::domain::errors::CleanpingError;
use cleanping_core::domain::local_server::LocalFix;
use cleanping_core::domain::models::Credential;

pub const ADD_FIRST_KEY: &str =
    "Add one: cleanping keys add --provider OpenAI (or run the guided setup: cleanping setup)";
pub const CHOOSE_A_KEY: &str = "Choose one: cleanping keys use NAME (or pass -c NAME).";
pub const SEE_KEYS: &str = "See: cleanping keys list";

/// Single-quoted for POSIX shells, so a name or URL cannot inject anything when pasted.
pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// A word that is safe to paste into a shell: left as it is when it is plain (a model name
/// such as `qwen2.5:7b`), quoted otherwise.
pub fn shell_word(value: &str) -> String {
    let plain = !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._:/@+-".contains(c));
    if plain {
        value.to_string()
    } else {
        shell_quote(value)
    }
}

/// One sentence: what to do about a local model server that is not ready.
pub fn local_fix(fix: LocalFix, model: &str) -> String {
    let pull = format!("ollama pull {}", shell_word(model));
    match fix {
        LocalFix::Install => format!(
            "Ollama does not seem to be installed. Get it from https://ollama.com/download, \
             then run: {pull}"
        ),
        LocalFix::Start => {
            "Ollama is installed but not running. Start it with: ollama serve".into()
        }
        LocalFix::PullModel => format!(
            "Ollama is running but does not have the model \u{201c}{model}\u{201d}. \
             Download it with: {pull}"
        ),
    }
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

    #[test]
    fn plain_words_are_left_alone_and_anything_else_is_quoted() {
        assert_eq!(shell_word("qwen2.5:7b"), "qwen2.5:7b");
        assert_eq!(
            shell_word("library/llama3:8b-instruct"),
            "library/llama3:8b-instruct"
        );
        assert_eq!(shell_word("my model"), "'my model'");
        assert_eq!(shell_word("a;touch x"), "'a;touch x'");
        assert_eq!(shell_word("$(id)"), "'$(id)'");
        assert_eq!(shell_word(""), "''");
    }

    #[test]
    fn each_fix_names_the_command_to_run_with_a_safe_model_name() {
        let pull = local_fix(LocalFix::PullModel, "qwen2.5:7b");
        assert!(pull.contains("ollama pull qwen2.5:7b"), "{pull}");
        let install = local_fix(LocalFix::Install, "qwen2.5:7b");
        assert!(install.contains("https://ollama.com/download"), "{install}");
        assert!(install.ends_with("ollama pull qwen2.5:7b"), "{install}");
        assert!(local_fix(LocalFix::Start, "m").contains("ollama serve"));
        let odd = local_fix(LocalFix::PullModel, "x;touch y");
        assert!(odd.contains("ollama pull 'x;touch y'"), "{odd}");
    }
}
