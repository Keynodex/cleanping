//! The body of an OpenAI-compatible chat-completions request, and how its reply is read.
//!
//! The draft goes inside `<draft>` tags with the fixed sentence after the saved prompt
//! ([`draft_frame`](crate::domain::draft_frame)), unless the draft holds a tag itself; a
//! [`provider_extras`] setting goes at the top level. Only this request body is framed: the
//! caller's text and prompt, which the history and the guards use, are never changed.

use serde_json::{json, Value};

use crate::application::ports::RewriteRequest;
use crate::domain::draft_frame::{frame_draft, frame_instructions};
use crate::domain::errors::Result;
use crate::domain::provider_extras::{provider_extras, RequestExtra};
use crate::infrastructure::chat_reply;

/// One request body, and whether its draft went out inside tags. Building it sends nothing.
pub(crate) struct ChatRequest {
    body: Value,
    framed: bool,
}

impl ChatRequest {
    /// The body for `request`: model, framed messages, and any extra for its provider.
    pub(crate) fn new(request: &RewriteRequest<'_>) -> Self {
        let (system, user, framed) = match frame_draft(request.text) {
            Some(user) => (frame_instructions(request.instructions), user, true),
            None => (
                request.instructions.to_string(),
                request.text.to_string(),
                false,
            ),
        };
        let mut body = json!({
            "model": request.model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user},
            ],
        });
        if let Some(extra) = provider_extras(request.api_url, request.model) {
            add_extra(&mut body, extra);
        }
        Self { body, framed }
    }

    /// The JSON to send.
    pub(crate) fn body(&self) -> &Value {
        &self.body
    }

    /// The edited text in the provider's reply `body`; echoed draft tags are removed only when
    /// this request framed the draft.
    pub(crate) fn edited_text(&self, body: &str) -> Result<String> {
        if self.framed {
            chat_reply::edited_draft(body)
        } else {
            chat_reply::edited_text(body)
        }
    }
}

fn add_extra(body: &mut Value, extra: RequestExtra) {
    match extra {
        RequestExtra::DeepSeekThinkingOff => body["thinking"] = json!({"type": "disabled"}),
    }
}

#[cfg(test)]
#[path = "chat_request_tests.rs"]
mod tests;
