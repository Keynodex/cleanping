//! Step 3: offer to test the connection with one tiny fixed request.

use cleanping_core::domain::errors::Result;
use cleanping_core::domain::models::Credential;

use super::ask::confirm;
use super::console::Console;
use super::tools::Tools;
use crate::connection;
use crate::services::Services;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tested {
    Passed,
    Failed,
    Skipped,
}

/// A failed test is a result to show, not an error of the setup: the key stays saved.
pub fn run(
    console: &mut dyn Console,
    tools: &dyn Tools,
    services: &Services,
    credential: &Credential,
) -> Result<Tested> {
    let question = "Step 3: test the connection now? It sends one tiny fixed request \
                    (never your text) and saves nothing.";
    if !confirm(console, question, true)? {
        console.say("Skipped. You can test any time with: cleanping keys test");
        return Ok(Tested::Skipped);
    }
    match tools.test(services, credential) {
        Ok(outcome) => {
            console.say(&connection::ok_line(credential, outcome));
            Ok(Tested::Passed)
        }
        Err(error) => {
            console.say(&error.to_string());
            Ok(Tested::Failed)
        }
    }
}

#[cfg(test)]
#[path = "check_tests.rs"]
mod tests;
