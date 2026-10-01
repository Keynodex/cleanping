//! Settings a request carries for one provider and model only. There is no user-facing switch
//! for these yet.

use super::validation::host_of;

/// A request setting added for a known provider and model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestExtra {
    /// Ask DeepSeek's flash models not to think before answering.
    DeepSeekThinkingOff,
}

/// The extra setting for this endpoint and model, if any. Only DeepSeek's own API
/// (`api.deepseek.com` exactly, in any letter case) with a model whose name starts with
/// `deepseek-flash` gets one; every other host, subdomain and model gets `None`.
pub fn provider_extras(api_url: &str, model: &str) -> Option<RequestExtra> {
    // Thinking was on by default: 13-44 s on a long text and cut-off or empty replies; with it
    // off, about 4.7 s and the best blind score (bake-off, 2026-10-01).
    let is_deepseek = host_of(api_url).is_some_and(|host| host.eq_ignore_ascii_case(DEEPSEEK_HOST));
    (is_deepseek && model.starts_with(FLASH_MODELS)).then_some(RequestExtra::DeepSeekThinkingOff)
}

const DEEPSEEK_HOST: &str = "api.deepseek.com";
const FLASH_MODELS: &str = "deepseek-flash";

#[cfg(test)]
#[path = "provider_extras_tests.rs"]
mod tests;
