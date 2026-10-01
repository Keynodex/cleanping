//! Asks GitHub's public API for CleanPing's newest release (ureq, rustls).
//!
//! The request carries only a `User-Agent: cleanping/<version>` header: no ids, keys, history or
//! text. Redirects are never followed, the reply is capped at [`MAX_REPLY_BYTES`] and its body
//! never appears in an error.

use std::time::Duration;

use serde_json::Value;
use ureq::config::AutoHeaderValue;

use crate::application::ports::{LatestRelease, ReleaseSource};
use crate::application::update_check::NOT_UNDERSTOOD;
use crate::domain::errors::{CleanpingError, Result};
use crate::domain::update_source::update_api_url;
use crate::domain::validation::is_local_url;
use crate::infrastructure::proxy_env;

/// A release description is a few kilobytes; anything bigger is refused, not read.
pub const MAX_REPLY_BYTES: u64 = 256 * 1024;

/// The whole request, from connecting to reading the reply, must finish within this.
pub const TIMEOUT: Duration = Duration::from_secs(15);

/// The environment variable that points the check at a fake server on this machine (tests only;
/// see [`update_api_url`] for what is accepted).
pub const URL_VARIABLE: &str = "CLEANPING_UPDATE_URL";

/// [`ReleaseSource`] backed by GitHub's `releases/latest` API.
pub struct GithubReleases {
    url: String,
    user_agent: String,
}

impl GithubReleases {
    /// Ask the address [`update_api_url`] resolves from [`URL_VARIABLE`], announcing
    /// `cleanping/<version>`.
    pub fn from_env(version: &str) -> Self {
        let url = update_api_url(std::env::var(URL_VARIABLE).ok().as_deref());
        Self {
            url,
            user_agent: format!("cleanping/{version}"),
        }
    }
}

fn failure(reason: &str) -> CleanpingError {
    CleanpingError::UpdateCheck(format!("Could not check for updates: {reason}"))
}

impl ReleaseSource for GithubReleases {
    fn latest(&self) -> Result<LatestRelease> {
        let mut builder = ureq::Agent::config_builder()
            .max_redirects(0)
            .max_redirects_will_error(false)
            .http_status_as_error(false)
            .timeout_global(Some(TIMEOUT))
            .user_agent(self.user_agent.as_str())
            .accept(AutoHeaderValue::None)
            .accept_encoding(AutoHeaderValue::None);
        if is_local_url(&self.url) {
            // Only the loopback test address gets here; it must stay on this machine.
            builder = builder.proxy(None);
        } else {
            builder = builder.https_only(true);
            proxy_env::refuse_unusable().map_err(|e| failure(&e.to_string()))?;
        }
        let agent = ureq::Agent::new_with_config(builder.build());
        let mut response = agent.get(&self.url).call().map_err(|error| match error {
            ureq::Error::Timeout(_) => failure("GitHub did not answer within 15 seconds."),
            _ => failure("GitHub could not be reached. Check your connection and try again."),
        })?;
        let code = response.status().as_u16();
        if (300..400).contains(&code) {
            return Err(failure(
                "GitHub redirected the request; refusing to follow it.",
            ));
        }
        if !(200..300).contains(&code) {
            return Err(failure(&format!("GitHub answered with HTTP {code}.")));
        }
        let body = response
            .body_mut()
            .with_config()
            .limit(MAX_REPLY_BYTES)
            .read_to_string()
            .map_err(|error| match error {
                ureq::Error::BodyExceedsLimit(_) => failure("the reply from GitHub was too large."),
                _ => CleanpingError::UpdateCheck(NOT_UNDERSTOOD.into()),
            })?;
        parse_release(&body).ok_or_else(|| CleanpingError::UpdateCheck(NOT_UNDERSTOOD.into()))
    }
}

/// The two fields used from GitHub's release JSON, or `None` when `tag_name` is not a string.
fn parse_release(body: &str) -> Option<LatestRelease> {
    let value: Value = serde_json::from_str(body).ok()?;
    Some(LatestRelease {
        tag_name: value.get("tag_name")?.as_str()?.to_string(),
        html_url: value
            .get("html_url")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}
