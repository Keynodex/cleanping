//! Proxy settings from the environment, checked so a broken one cannot quietly become a direct
//! connection.
//!
//! The HTTP client ignores a proxy value it cannot parse, and connects directly when a SOCKS
//! proxy is set but SOCKS is not built in. Either would send your text somewhere you did not
//! expect, so a proxy variable that is set but unusable is an error here.

use ureq::{Proxy, ProxyProtocol};

use crate::domain::errors::{CleanpingError, Result};

/// In the order the HTTP client reads them.
const VARIABLES: [&str; 6] = [
    "ALL_PROXY",
    "all_proxy",
    "HTTPS_PROXY",
    "https_proxy",
    "HTTP_PROXY",
    "http_proxy",
];

/// The first proxy variable that is set to something this program cannot use. Only the name
/// is returned: a proxy address may carry a password.
pub fn unusable_variable(get: impl Fn(&str) -> Option<String>) -> Option<&'static str> {
    VARIABLES.into_iter().find(|name| {
        get(name)
            .filter(|value| !value.trim().is_empty())
            .is_some_and(|value| match Proxy::new(&value) {
                Ok(proxy) => {
                    !matches!(proxy.protocol(), ProxyProtocol::Http | ProxyProtocol::Https)
                }
                Err(_) => true,
            })
    })
}

/// Refuse to go on when the real environment holds an unusable proxy setting.
pub fn refuse_unusable() -> Result<()> {
    match unusable_variable(|name| std::env::var(name).ok()) {
        Some(name) => Err(CleanpingError::Rewrite(format!(
            "Your proxy setting {name} cannot be used (only http:// and https:// proxies work), \
             so nothing was sent. Fix or unset it."
        ))),
        None => Ok(()),
    }
}

#[cfg(test)]
#[path = "proxy_env_tests.rs"]
mod tests;
