"""Provider presets for the API key dialog: pick one, the URL fills in."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class ProviderPreset:
    label: str
    api_url: str
    default_model: str


PROVIDER_PRESETS: tuple[ProviderPreset, ...] = (
    ProviderPreset(
        "DeepSeek",
        "https://api.deepseek.com/v1/chat/completions",
        "deepseek-chat",
    ),
    ProviderPreset(
        "OpenAI",
        "https://api.openai.com/v1/chat/completions",
        "gpt-4o-mini",
    ),
    ProviderPreset(
        "OpenRouter",
        "https://openrouter.ai/api/v1/chat/completions",
        "openai/gpt-4o-mini",
    ),
    ProviderPreset(
        "Ollama (local)",
        "http://127.0.0.1:11434/v1/chat/completions",
        "qwen2.5:7b",
    ),
    ProviderPreset("Custom", "", ""),
)

PRESET_LABELS = tuple(preset.label for preset in PROVIDER_PRESETS)


def preset_for(label: str) -> ProviderPreset | None:
    for preset in PROVIDER_PRESETS:
        if preset.label == label:
            return preset
    return None


def preset_label_for_url(api_url: str) -> str:
    """The preset whose URL matches exactly; anything else is "Custom"."""
    for preset in PROVIDER_PRESETS:
        if preset.api_url and preset.api_url == api_url:
            return preset.label
    return "Custom"
