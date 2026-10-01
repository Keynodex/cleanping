use super::*;

const DEEPSEEK: &str = "https://api.deepseek.com/v1/chat/completions";

#[test]
fn deepseek_flash_on_deepseek_turns_thinking_off() {
    for (url, model) in [
        (DEEPSEEK, "deepseek-flash"),
        (DEEPSEEK, "deepseek-flash-2"),
        (
            "https://api.deepseek.com/chat/completions",
            "deepseek-flash",
        ),
        (
            "https://API.DeepSeek.COM/v1/chat/completions",
            "deepseek-flash",
        ),
        (
            "  https://api.deepseek.com:443/v1/chat/completions ",
            "deepseek-flash",
        ),
    ] {
        assert_eq!(
            provider_extras(url, model),
            Some(RequestExtra::DeepSeekThinkingOff),
            "{url} {model}"
        );
    }
}

#[test]
fn other_deepseek_models_are_unchanged() {
    for model in [
        "deepseek-chat",
        "deepseek-reasoner",
        "flash",
        "my-deepseek-flash",
        "",
    ] {
        assert_eq!(provider_extras(DEEPSEEK, model), None, "{model}");
    }
}

#[test]
fn the_model_name_is_matched_as_written() {
    assert_eq!(provider_extras(DEEPSEEK, "DeepSeek-Flash"), None);
}

#[test]
fn other_hosts_get_nothing_even_for_a_flash_model() {
    for url in [
        "https://api.deepseek.com.evil.example/v1/chat/completions",
        "https://evil.example/api.deepseek.com/v1/chat/completions",
        "https://eu.api.deepseek.com/v1/chat/completions",
        "https://deepseek.com/v1/chat/completions",
        "https://api-deepseek.com/v1/chat/completions",
        "https://api.openai.com/v1/chat/completions",
        "https://openrouter.ai/api/v1/chat/completions",
        "http://127.0.0.1:11434/v1/chat/completions",
        "not a url",
        "",
    ] {
        assert_eq!(provider_extras(url, "deepseek-flash"), None, "{url}");
    }
}

#[test]
fn a_user_name_that_looks_like_the_host_does_not_count() {
    assert_eq!(
        provider_extras(
            "https://api.deepseek.com@evil.example/v1/chat/completions",
            "deepseek-flash"
        ),
        None
    );
}
