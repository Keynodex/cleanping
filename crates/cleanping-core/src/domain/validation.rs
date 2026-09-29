//! Pure invariants for provider URLs and credential fields.

use std::net::{Ipv4Addr, Ipv6Addr};

use url::{Host, Url};

use super::errors::{CleanpingError, Result};
use super::models::CredentialInput;
use super::sanitize::is_plain_line;

const MSG_NEEDS_HTTPS: &str = "API URL must use HTTPS (HTTP is allowed only for localhost).";
/// Longest API key accepted; real keys are far shorter.
pub const MAX_API_KEY_BYTES: usize = 4096;

const MSG_BAD_URL: &str = "API URL must be a valid URL without embedded credentials.";

fn is_local_host(host: &Host<&str>) -> bool {
    match host {
        Host::Domain(name) => *name == "localhost",
        Host::Ipv4(addr) => *addr == Ipv4Addr::LOCALHOST,
        Host::Ipv6(addr) => *addr == Ipv6Addr::LOCALHOST,
    }
}

/// True when the URL points at this machine (no key needs to travel).
pub fn is_local_url(url: &str) -> bool {
    Url::parse(url.trim())
        .ok()
        .and_then(|parsed| parsed.host().map(|host| is_local_host(&host)))
        .unwrap_or(false)
}

/// The host part of a URL, for showing where text is going.
pub fn host_of(url: &str) -> Option<String> {
    let parsed = Url::parse(url.trim()).ok()?;
    parsed.host_str().map(str::to_string)
}

/// True when both URLs have the same scheme, host and port (paths may differ).
pub fn same_origin(a: &str, b: &str) -> bool {
    match (Url::parse(a.trim()), Url::parse(b.trim())) {
        (Ok(a), Ok(b)) => {
            a.scheme() == b.scheme()
                && a.host() == b.host()
                && a.port_or_known_default() == b.port_or_known_default()
        }
        _ => false,
    }
}

/// `scheme://` followed by nothing or another `/` has no host. The `url` crate would
/// quietly treat `https:///v1/x` as host `v1`, so this is checked on the raw text.
fn has_empty_authority(text: &str) -> bool {
    text.split_once("://")
        .is_some_and(|(_, rest)| rest.is_empty() || rest.starts_with('/'))
}

/// Return the URL trimmed when it is safe to send an API key to.
pub fn validate_api_url(url: &str) -> Result<String> {
    let normalized = url.trim();
    let bad = |message: &str| CleanpingError::Validation(message.to_string());
    let Ok(parsed) = Url::parse(normalized) else {
        let looks_https = normalized.to_ascii_lowercase().starts_with("https://");
        return Err(bad(if looks_https {
            MSG_BAD_URL
        } else {
            MSG_NEEDS_HTTPS
        }));
    };
    let local_http = parsed.scheme() == "http" && parsed.host().is_some_and(|h| is_local_host(&h));
    if parsed.scheme() != "https" && !local_http {
        return Err(bad(MSG_NEEDS_HTTPS));
    }
    let has_credentials = !parsed.username().is_empty() || parsed.password().is_some();
    if parsed.host().is_none() || has_credentials || has_empty_authority(normalized) {
        return Err(bad(MSG_BAD_URL));
    }
    // The canonical form: what the HTTP client will parse is exactly what was checked here.
    Ok(parsed.to_string())
}

/// Blank is allowed (it means "keep the saved key"). Otherwise only plain visible ASCII, so a
/// pasted smart quote or stray space fails here, not later as a confusing network error.
fn validate_api_key(key: &str) -> Result<String> {
    if key.len() > MAX_API_KEY_BYTES {
        return Err(CleanpingError::Validation(format!(
            "API key is too long (limit {MAX_API_KEY_BYTES} bytes)."
        )));
    }
    if !key.chars().all(|c| c.is_ascii_graphic()) {
        return Err(CleanpingError::Validation(
            "API key must be plain visible characters (no spaces, control characters or smart quotes)."
                .into(),
        ));
    }
    Ok(key.to_string())
}

/// Validate name, URL and model; return a whitespace-normalized copy.
pub fn validate_credential_fields(draft: CredentialInput) -> Result<CredentialInput> {
    let name = draft.name.trim();
    if name.is_empty() {
        return Err(CleanpingError::Validation("Give this key a name.".into()));
    }
    let model = draft.model.trim();
    if model.is_empty() {
        return Err(CleanpingError::Validation(
            "model must not be empty.".into(),
        ));
    }
    // Both are printed in lists and messages, so they must not carry control or hidden characters.
    for (what, value) in [("name", name), ("model", model)] {
        if !is_plain_line(value) {
            return Err(CleanpingError::Validation(format!(
                "The {what} must be plain text on one line."
            )));
        }
    }
    let api_key = validate_api_key(draft.api_key.trim())?;
    Ok(CredentialInput {
        name: name.to_string(),
        api_url: validate_api_url(&draft.api_url)?,
        model: model.to_string(),
        api_key,
    })
}

#[cfg(test)]
#[path = "validation_tests.rs"]
mod tests;
