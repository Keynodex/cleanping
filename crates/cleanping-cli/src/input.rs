//! Where the text to rewrite (or a key, or a prompt) comes from: arguments or a pipe.

use std::io::{BufRead, IsTerminal, Read};

use cleanping_core::domain::errors::{CleanpingError, Result};
use cleanping_core::domain::validation::MAX_API_KEY_BYTES;

/// Larger inputs are refused before any request is made (cost and abuse control).
pub const MAX_INPUT_BYTES: usize = 200_000;

fn invalid(message: &str) -> CleanpingError {
    CleanpingError::Validation(message.to_string())
}

fn too_long() -> CleanpingError {
    invalid(&format!(
        "Input is too long (limit {MAX_INPUT_BYTES} bytes)."
    ))
}

/// Everything on stdin, capped. Refuses invalid UTF-8 and oversized input.
pub fn read_stdin() -> Result<String> {
    let mut bytes = Vec::new();
    std::io::stdin()
        .lock()
        .take(MAX_INPUT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| invalid("Could not read the input."))?;
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(too_long());
    }
    String::from_utf8(bytes).map_err(|_| invalid("Input is not valid UTF-8 text."))
}

/// One line from stdin (a key), capped so a runaway pipe cannot fill memory. The read cap
/// leaves room for a `\r\n` line ending on top of the longest allowed key.
pub fn read_key_line() -> Result<String> {
    let mut line = String::new();
    std::io::stdin()
        .lock()
        .take(MAX_API_KEY_BYTES as u64 + 3)
        .read_line(&mut line)
        .map_err(|_| invalid("Could not read the key."))?;
    let key = line.trim();
    if key.len() > MAX_API_KEY_BYTES {
        return Err(invalid(&format!(
            "API key is too long (limit {MAX_API_KEY_BYTES} bytes)."
        )));
    }
    Ok(key.to_string())
}

pub fn stdin_is_a_terminal() -> bool {
    std::io::stdin().is_terminal()
}

/// Text from the arguments, else from a pipe. Trimmed and never empty.
pub fn text_from(args: &[String]) -> Result<String> {
    let raw = if !args.is_empty() {
        args.join(" ")
    } else if stdin_is_a_terminal() {
        return Err(invalid(
            "Nothing to rewrite. Pass text, or pipe it in: echo 'rough text' | cleanping",
        ));
    } else {
        read_stdin()?
    };
    if raw.len() > MAX_INPUT_BYTES {
        return Err(too_long());
    }
    let text = raw.trim();
    if text.is_empty() {
        return Err(invalid("Nothing to rewrite."));
    }
    Ok(text.to_string())
}
