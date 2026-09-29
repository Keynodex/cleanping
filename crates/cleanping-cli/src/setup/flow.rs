//! Puts the steps together: a guided first run, and a small menu when a key is already saved.

use cleanping_core::domain::errors::Result;
use cleanping_core::domain::models::Credential;

use super::ask::choose;
use super::check::{self, Tested};
use super::console::Console;
use super::tools::Tools;
use super::usage::usage_lines;
use super::{provider, system_prompt};
use crate::args::Shell;
use crate::services::Services;

/// `shell` is the user's shell, if known, so the closing screen shows the right profile file.
pub fn run(
    console: &mut dyn Console,
    tools: &dyn Tools,
    services: &Services,
    shell: Option<Shell>,
) -> Result<()> {
    console.say("CleanPing setup");
    console.say("");
    if services.credentials.list()?.is_empty() {
        first_run(console, tools, services, shell)
    } else {
        menu(console, tools, services, shell)
    }
}

fn first_run(
    console: &mut dyn Console,
    tools: &dyn Tools,
    services: &Services,
    shell: Option<Shell>,
) -> Result<()> {
    console.say("This takes about a minute. Nothing is sent to a provider until you test the");
    console.say("connection.");
    console.say("");
    let credential = provider::run(console, tools, services)?;
    console.say("");
    system_prompt::run(console, services)?;
    console.say("");
    let tested = check::run(console, tools, services, &credential)?;
    console.say("");
    console.say(match tested {
        Tested::Passed => "All set.",
        Tested::Failed => {
            "Almost there: the connection test failed (see above). Fix that, then run \
             cleanping setup again or cleanping keys test."
        }
        Tested::Skipped => "Saved, but not tested yet. Run cleanping keys test when you are ready.",
    });
    console.say("");
    show_usage(console, shell);
    Ok(())
}

fn menu(
    console: &mut dyn Console,
    tools: &dyn Tools,
    services: &Services,
    shell: Option<Shell>,
) -> Result<()> {
    loop {
        let current = current_credential(services);
        let now = match &current {
            Some(c) => format!("now: \u{201c}{}\u{201d}, model {}", c.name, c.model),
            None => "none selected".to_string(),
        };
        let options = [
            format!("AI provider and key ({now})"),
            "System prompt".to_string(),
            "Test the connection".to_string(),
            "How to use CleanPing".to_string(),
            "Quit".to_string(),
        ];
        let quit = options.len() - 1;
        let title = "What would you like to change?";
        match choose(console, title, &options, Some(quit))? {
            0 => {
                provider::run(console, tools, services)?;
            }
            1 => system_prompt::run(console, services)?,
            2 => match &current {
                Some(credential) => {
                    check::run(console, tools, services, credential)?;
                }
                None => console.say(
                    "Choose your AI first (option 1): several keys are saved and none is selected.",
                ),
            },
            3 => show_usage(console, shell),
            _ => return Ok(()),
        }
        console.say("");
    }
}

fn current_credential(services: &Services) -> Option<Credential> {
    let selected = services.state.selected_credential_id().ok()?;
    services.credentials.resolve(None, selected).ok()
}

fn show_usage(console: &mut dyn Console, shell: Option<Shell>) {
    for line in usage_lines(shell) {
        console.say(&line);
    }
}

#[cfg(test)]
#[path = "flow_tests.rs"]
mod tests;
