//! Provider presets: pick one, the URL fills in.

#[derive(Debug, PartialEq, Eq)]
pub struct ProviderPreset {
    pub label: &'static str,
    pub api_url: &'static str,
    pub default_model: &'static str,
}

pub const PROVIDER_PRESETS: &[ProviderPreset] = &[
    ProviderPreset {
        label: "DeepSeek",
        api_url: "https://api.deepseek.com/v1/chat/completions",
        default_model: "deepseek-chat",
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

pub fn preset_labels() -> Vec<&'static str> {
    PROVIDER_PRESETS.iter().map(|p| p.label).collect()
}

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
