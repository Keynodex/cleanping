//! Where the update check asks, and which release page address may be shown to the user.

use url::Url;

use super::validation::is_local_url;

/// GitHub's public API for the newest published release of CleanPing.
pub const LATEST_RELEASE_API: &str =
    "https://api.github.com/repos/Keynodex/cleanping/releases/latest";

/// The page shown when the reply's own page address cannot be trusted.
pub const RELEASES_PAGE: &str = "https://github.com/Keynodex/cleanping/releases/latest";

/// A release page address from the reply is shown only when it starts with this.
pub const RELEASE_PAGE_PREFIX: &str = "https://github.com/Keynodex/cleanping/";

/// A page address longer than this is not shown (the real ones are about 60 characters).
const MAX_PAGE_URL_CHARS: usize = 200;

/// The address to ask: `override_url` only when it is plain `http://` to this machine
/// (`localhost`, `127.0.0.1` or `[::1]`, without a user name or password), so tests can use a
/// fake server. Anything else is ignored and [`LATEST_RELEASE_API`] is used, so a setting can
/// never send the check to another computer.
pub fn update_api_url(override_url: Option<&str>) -> String {
    override_url
        .and_then(loopback_http)
        .unwrap_or_else(|| LATEST_RELEASE_API.to_string())
}

/// `url` in canonical form when it is plain `http://` to this machine, else `None`.
fn loopback_http(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    let plain = parsed.scheme() == "http" && parsed.username().is_empty();
    (plain && parsed.password().is_none() && is_local_url(url)).then(|| parsed.to_string())
}

/// The release page to print: `html_url` from the reply when it is a plain address under
/// [`RELEASE_PAGE_PREFIX`], otherwise [`RELEASES_PAGE`]. The result never holds a space or a
/// control character.
pub fn release_page(html_url: Option<&str>) -> String {
    html_url
        .filter(|page| page.starts_with(RELEASE_PAGE_PREFIX))
        .filter(|page| page.chars().count() <= MAX_PAGE_URL_CHARS)
        .filter(|page| page.chars().all(|c| c.is_ascii_graphic()))
        .unwrap_or(RELEASES_PAGE)
        .to_string()
}

#[cfg(test)]
#[path = "update_source_tests.rs"]
mod tests;
