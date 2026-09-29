//! For a provider on this computer (Ollama): say whether it is ready, and offer to fetch the model.

use cleanping_core::domain::errors::Result;
use cleanping_core::domain::local_server::{
    diagnose, is_default_ollama_url, LocalFix, LocalServer,
};
use cleanping_core::domain::models::Credential;

use super::ask::confirm;
use super::console::Console;
use super::tools::Tools;
use crate::hints;

/// For a server on this computer: say whether it is ready, and offer to fetch the model.
pub fn prepare(
    console: &mut dyn Console,
    tools: &dyn Tools,
    credential: &Credential,
) -> Result<()> {
    let Some(server) = tools.probe(&credential.api_url) else {
        return Ok(());
    };
    let model = &credential.model;
    let default_address = is_default_ollama_url(&credential.api_url);
    match diagnose(&server, model, default_address) {
        Some(LocalFix::PullModel) => offer_download(console, tools, model),
        Some(fix) => {
            console.say(&format!(
                "{} After that, run cleanping setup again.",
                hints::local_fix(fix, model)
            ));
            Ok(())
        }
        None => {
            if matches!(server, LocalServer::Running { .. }) {
                console.say(&format!(
                    "Ollama is running and has \u{201c}{model}\u{201d}."
                ));
            }
            Ok(())
        }
    }
}

fn offer_download(console: &mut dyn Console, tools: &dyn Tools, model: &str) -> Result<()> {
    if model.starts_with('-') {
        console.say("The model name looks like an option, so setup will not pass it to ollama.");
        return Ok(());
    }
    let command = format!("ollama pull {}", hints::shell_word(model));
    console.say(&format!(
        "Ollama is running but does not have the model \u{201c}{model}\u{201d}."
    ));
    let question = "Download it now with `ollama pull`? It can be several gigabytes.";
    if !confirm(console, question, false)? {
        console.say(&format!("You can download it later with: {command}"));
    } else if tools.pull(model) {
        console.say("Downloaded.");
    } else {
        console.say(&format!(
            "The download did not finish. You can try again later with: {command}"
        ));
    }
    Ok(())
}
