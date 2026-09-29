//! `cleanping keys test`: does a saved provider answer?

use std::time::Duration;

use cleanping_core::application::connection_check::{CheckOutcome, ConnectionCheck};
use cleanping_core::domain::errors::{CleanpingError, Result};
use cleanping_core::domain::local_server::{diagnose, is_default_ollama_url};
use cleanping_core::domain::models::Credential;
use cleanping_core::infrastructure::http_rewriter::OpenAiRewriter;
use cleanping_core::infrastructure::ollama;

use crate::hints;
use crate::output;
use crate::rewrite::pick_credential;
use crate::services::Services;

/// A cold local model can need a while to load; a hung remote server should not hold the
/// terminal for the full three minutes a rewrite allows.
const TEST_TIMEOUT: Duration = Duration::from_secs(60);

pub fn run(services: &Services, name: Option<&str>) -> Result<()> {
    let credential = pick_credential(name, services)?;
    let outcome = test(services, &credential)?;
    output::line(&ok_line(&credential, outcome))
}

/// One tiny request to the provider. A failure comes back with the key's name and, for a
/// local model server, what to do about it.
pub fn test(services: &Services, credential: &Credential) -> Result<CheckOutcome> {
    let check = ConnectionCheck::new(OpenAiRewriter::new(TEST_TIMEOUT), services.secrets());
    check
        .run(credential)
        .map_err(|error| explain(error, credential))
}

pub fn ok_line(credential: &Credential, outcome: CheckOutcome) -> String {
    format!(
        "OK: \u{201c}{}\u{201d} answered in {} ms (model {}).",
        credential.name, outcome.duration_ms, credential.model
    )
}

/// The failure with the key's name, and, for a local model server, what to do about it.
fn explain(error: CleanpingError, credential: &Credential) -> CleanpingError {
    match error {
        CleanpingError::Rewrite(message) => CleanpingError::Rewrite(format!(
            "The test with \u{201c}{}\u{201d} failed: {message}{}",
            credential.name,
            local_advice(credential)
        )),
        other => hints::with_add_hint(other, credential),
    }
}

/// A leading space and one sentence when the provider is a local server that is not ready.
fn local_advice(credential: &Credential) -> String {
    let Some(server) = ollama::probe(&credential.api_url) else {
        return String::new();
    };
    let default_address = is_default_ollama_url(&credential.api_url);
    diagnose(&server, &credential.model, default_address)
        .map(|fix| format!(" {}", hints::local_fix(fix, &credential.model)))
        .unwrap_or_default()
}
