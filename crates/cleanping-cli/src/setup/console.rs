//! How the wizard talks to the user. Everything goes through `Console`, so the whole wizard
//! can be tested with scripted answers.

use std::io::{BufRead, Write};

use cleanping_core::domain::errors::{CleanpingError, Result};

pub trait Console {
    fn say(&mut self, text: &str);
    /// One line the user typed, without the line break. `Err` when the input has ended.
    fn ask(&mut self, question: &str) -> Result<String>;
    /// Like `ask`, but nothing is shown while the user types (for API keys).
    fn ask_secret(&mut self, question: &str) -> Result<String>;
}

pub fn stopped() -> CleanpingError {
    CleanpingError::Validation("Setup stopped: no answer was given.".into())
}

pub fn too_many_bad_answers() -> CleanpingError {
    CleanpingError::Validation("Setup stopped: too many unclear answers in a row.".into())
}

/// The real terminal: questions on stdout, answers from stdin, keys through a hidden prompt.
pub struct StdConsole;

impl Console for StdConsole {
    fn say(&mut self, text: &str) {
        let _ = crate::output::line(text);
    }

    fn ask(&mut self, question: &str) -> Result<String> {
        let mut out = std::io::stdout().lock();
        let _ = write!(out, "{question}");
        let _ = out.flush();
        let mut line = String::new();
        match std::io::stdin().lock().read_line(&mut line) {
            Ok(0) | Err(_) => Err(stopped()),
            Ok(_) => Ok(line.trim_end_matches(['\r', '\n']).to_string()),
        }
    }

    fn ask_secret(&mut self, question: &str) -> Result<String> {
        crate::secret_input::read_secret(question)
    }
}
