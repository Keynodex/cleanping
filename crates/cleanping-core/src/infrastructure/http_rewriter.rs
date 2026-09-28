//! HTTP adapter for OpenAI-compatible chat completions (ureq, rustls).
//!
//! Safety rules: the URL gate runs first, redirects are never followed (they could carry
//! the key and the text to another host), and server response bodies are never echoed.

use std::time::Duration;

use serde_json::{json, Value};

use crate::application::ports::{RewriteRequest, Rewriter};
use crate::domain::errors::{CleanpingError, Result};
use crate::domain::sanitize::clean_reply;
use crate::domain::validation::{is_local_url, validate_api_url};
use crate::infrastructure::proxy_env;

/// A rewrite is short text; anything bigger is refused, not read into memory.
pub const MAX_RESPONSE_BYTES: u64 = 1_000_000;

/// Cold local models can take a while to load.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(180);

pub struct OpenAiRewriter {
    timeout: Duration,
}

impl OpenAiRewriter {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl Default for OpenAiRewriter {
    fn default() -> Self {
        Self::new(DEFAULT_TIMEOUT)
    }
}

fn rewrite_error(message: impl Into<String>) -> CleanpingError {
    CleanpingError::Rewrite(message.into())
}

impl Rewriter for OpenAiRewriter {
    fn rewrite(&self, request: &RewriteRequest<'_>) -> Result<String> {
        let RewriteRequest {
            text,
            instructions,
            api_url,
            api_key,
            model,
        } = *request;
        let url = validate_api_url(api_url)?; // never send the key to a URL the domain rejects
        if api_key.chars().any(char::is_control) {
            return Err(CleanpingError::Validation(
                "API key contains invalid characters.".into(),
            ));
        }
        let mut builder = ureq::Agent::config_builder()
            .max_redirects(0)
            .max_redirects_will_error(false)
            .http_status_as_error(false)
            .timeout_global(Some(self.timeout));
        if is_local_url(&url) {
            // A loopback address must stay on this machine, whatever HTTP_PROXY says.
            builder = builder.proxy(None);
        } else {
            proxy_env::refuse_unusable()?;
        }
        let config = builder.build();
        let agent = ureq::Agent::new_with_config(config);
        let payload = json!({
            "model": model,
            "messages": [
                {"role": "system", "content": instructions},
                {"role": "user", "content": text},
            ],
        });
        let mut request = agent.post(&url);
        if !api_key.is_empty() {
            request = request.header("Authorization", format!("Bearer {api_key}"));
        }
        let mut response = request
            .send_json(&payload)
            .map_err(|e| rewrite_error(format!("Could not reach the API: {e}.")))?;
        let code = response.status().as_u16();
        if (300..400).contains(&code) {
            return Err(rewrite_error(format!(
                "API redirected the request (HTTP {code}); refusing to follow it."
            )));
        }
        if !(200..300).contains(&code) {
            // Never echo server bodies; they can contain private prompts.
            return Err(rewrite_error(format!("API returned HTTP {code}.")));
        }
        let body = response
            .body_mut()
            .with_config()
            .limit(MAX_RESPONSE_BYTES)
            .read_to_string()
            .map_err(|error| match error {
                ureq::Error::BodyExceedsLimit(_) => rewrite_error(format!(
                    "API response was too large (limit {MAX_RESPONSE_BYTES} bytes)."
                )),
                _ => rewrite_error("API response could not be read."),
            })?;
        let edited = serde_json::from_str::<Value>(&body)
            .ok()
            .and_then(|v| {
                v["choices"][0]["message"]["content"]
                    .as_str()
                    .map(clean_reply)
            })
            .ok_or_else(|| rewrite_error("API response did not contain edited text."))?;
        if edited.is_empty() {
            return Err(rewrite_error(
                "API returned an empty edit; nothing was copied.",
            ));
        }
        Ok(edited)
    }
}

#[cfg(test)]
#[path = "http_rewriter_tests.rs"]
mod tests;
