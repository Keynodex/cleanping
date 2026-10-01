//! Reads the edited text out of an OpenAI-compatible chat completion body.
//!
//! The body is untrusted: only the first choice is used, its text is cleaned, and no part of
//! the body ever goes into an error message.

use serde_json::Value;

use crate::domain::errors::{CleanpingError, Result};
use crate::domain::sanitize::clean_reply;

/// The cleaned text of the first choice, or a `Rewrite` error with a safe message.
pub(crate) fn edited_text(body: &str) -> Result<String> {
    let reply: Value = serde_json::from_str(body).map_err(|_| no_edited_text())?;
    let choice = &reply["choices"][0];
    let edited = choice["message"]["content"]
        .as_str()
        .map(clean_reply)
        .ok_or_else(no_edited_text)?;
    if edited.is_empty() {
        return Err(CleanpingError::Rewrite(
            "API returned an empty edit; nothing was copied.".into(),
        ));
    }
    Ok(edited)
}

fn no_edited_text() -> CleanpingError {
    CleanpingError::Rewrite("API response did not contain edited text.".into())
}

#[cfg(test)]
#[path = "chat_reply_tests.rs"]
mod tests;
