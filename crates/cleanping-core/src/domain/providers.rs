//! Provider presets: pick one, the URL fills in.

/// A known OpenAI-compatible provider. Picking one fills in its URL and a starting model.
#[derive(Debug, PartialEq, Eq)]
pub struct ProviderPreset {
    /// Name shown to the user and matched by [`preset_for`].
    pub label: &'static str,
    /// Its chat-completions endpoint; empty for `Custom`.
    pub api_url: &'static str,
    /// Model suggested when the provider is picked; empty for `Custom`.
    pub default_model: &'static str,
}

/// The known providers, in the order the Python release showed them; `Custom` (no URL) is last.
pub const PROVIDER_PRESETS: &[ProviderPreset] = &[
    ProviderPreset {
        label: "DeepSeek",
        api_url: "https://api.deepseek.com/v1/chat/completions",
        default_model: "deepseek-flash",
    },
    ProviderPreset {
        label: "OpenAI",
        api_url: "https://api.openai.com/v1/chat/completions",
        default_model: "gpt-4o-mini",
    },
    ProviderPreset {
        label: "OpenRouter",
        api_url: "https://openrouter.ai/api/v1/chat/completions",
        default_model: "openai/gpt-4o-mini",
    },
    ProviderPreset {
        label: "Ollama (local)",
        api_url: "http://127.0.0.1:11434/v1/chat/completions",
        default_model: "qwen2.5:7b",
    },
    ProviderPreset {
        label: "Custom",
        api_url: "",
        default_model: "",
    },
];

/// Every preset's label, in order.
pub fn preset_labels() -> Vec<&'static str> {
    PROVIDER_PRESETS.iter().map(|p| p.label).collect()
}

/// The preset with exactly this label (case-sensitive).
pub fn preset_for(label: &str) -> Option<&'static ProviderPreset> {
    PROVIDER_PRESETS.iter().find(|p| p.label == label)
}

/// The preset whose URL matches exactly; anything else is "Custom".
pub fn preset_label_for_url(api_url: &str) -> &'static str {
    PROVIDER_PRESETS
        .iter()
        .find(|p| !p.api_url.is_empty() && p.api_url == api_url)
        .map_or("Custom", |p| p.label)
}

#[cfg(test)]
#[path = "providers_tests.rs"]
mod tests;
