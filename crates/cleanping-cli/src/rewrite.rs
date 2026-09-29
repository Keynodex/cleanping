//! The main command: rewrite text and print only the result on stdout.

use cleanping_core::application::polisher::{PolishResult, PolishText};
use cleanping_core::application::ports::RunRepository;
use cleanping_core::domain::errors::{CleanpingError, Result};
use cleanping_core::domain::models::Credential;
use cleanping_core::infrastructure::http_rewriter::OpenAiRewriter;
use cleanping_core::infrastructure::sqlite_repositories::SqliteRunRepository;

use crate::args::Cli;
use crate::clipboard;
use crate::guard;
use crate::hints;
use crate::input;
use crate::no_history::NoHistory;
use crate::output;
use crate::services::Services;

/// The key named by `--credential`, else the selected one, else the only one. Never a guess.
pub fn pick_credential(name: Option<&str>, services: &Services) -> Result<Credential> {
    let selected = services.state.selected_credential_id()?;
    services
        .credentials
        .resolve(name, selected)
        .map_err(|error| match error {
            CleanpingError::MissingCredential(message) => {
                let none_saved = services
                    .credentials
                    .list()
                    .map_or(true, |all| all.is_empty());
                let hint = if none_saved {
                    hints::ADD_FIRST_KEY
                } else {
                    hints::CHOOSE_A_KEY
                };
                CleanpingError::MissingCredential(format!("{message} {hint}"))
            }
            CleanpingError::NotFound(message) => {
                CleanpingError::NotFound(format!("{message} {}", hints::SEE_KEYS))
            }
            other => other,
        })
}

fn polish_with<R: RunRepository>(
    cli: &Cli,
    runs: R,
    services: &Services,
    text: &str,
    credential: &Credential,
) -> Result<PolishResult> {
    let instructions = services.prompts.current()?;
    let mut polisher = PolishText::new(OpenAiRewriter::default(), runs, services.secrets());
    if cli.keep_shape {
        polisher = polisher.keeping_shape();
    }
    polisher.run(text, &instructions, credential)
}

fn polish(
    cli: &Cli,
    services: &Services,
    text: &str,
    credential: &Credential,
) -> Result<PolishResult> {
    if cli.no_history {
        polish_with(cli, NoHistory, services, text, credential)
    } else {
        let runs = SqliteRunRepository::new(services.db.clone());
        polish_with(cli, runs, services, text, credential)
    }
}

pub fn run(cli: &Cli, services: &Services) -> Result<()> {
    let text = input::text_from(&cli.text)?;
    if cli.refuse_secrets {
        guard::refuse_if_secret(&text)?;
    }
    let credential = pick_credential(cli.credential.as_deref(), services)?;
    let result = polish(cli, services, &text, &credential)
        .map_err(|error| hints::with_add_hint(error, &credential))?;
    match (result.output_text, result.error_message) {
        (Some(output), _) => {
            output::line(&output)?;
            if cli.copy && !clipboard::copy(&output) {
                output::warn(&format!(
                    "warning: could not copy to the clipboard (tried {}).",
                    clipboard::TOOL_NAMES
                ));
            }
            Ok(())
        }
        (None, message) => Err(CleanpingError::Rewrite(
            message.unwrap_or_else(|| "The API returned nothing.".into()),
        )),
    }
}
