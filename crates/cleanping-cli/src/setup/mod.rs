//! `cleanping setup`: a guided first run (pick the AI, save the key, pick a system prompt,
//! test the connection), and a small menu when run again.

mod ask;
mod check;
mod console;
mod flow;
mod local_model;
mod provider;
mod system_prompt;
#[cfg(test)]
mod testing;
mod tools;
mod usage;

use cleanping_core::domain::errors::{CleanpingError, Result};

use crate::input;
use crate::services::Services;

pub fn run(services: &Services) -> Result<()> {
    if !input::stdin_is_a_terminal() {
        return Err(CleanpingError::Validation(
            "cleanping setup asks questions, so it needs a terminal. Without one, save a key \
             with: cleanping keys add --provider OpenAI --key-stdin"
                .into(),
        ));
    }
    let shell = std::env::var("SHELL")
        .ok()
        .and_then(|path| usage::shell_from_path(&path));
    flow::run(&mut console::StdConsole, &tools::RealTools, services, shell)
}
