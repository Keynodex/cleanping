use super::*;

#[test]
fn known_url_maps_to_its_preset_label() {
    let preset = &PROVIDER_PRESETS[0];
    assert_eq!(preset_label_for_url(preset.api_url), preset.label);
}

#[test]
fn unknown_and_empty_urls_are_custom() {
    assert_eq!(
        preset_label_for_url("https://example.test/v1/chat/completions"),
        "Custom"
    );
    assert_eq!(preset_label_for_url(""), "Custom");
}

#[test]
fn labels_match_the_python_release_in_order() {
    assert_eq!(
        preset_labels(),
        [
            "DeepSeek",
            "OpenAI",
            "OpenRouter",
            "Ollama (local)",
            "Custom"
        ]
    );
}

#[test]
fn preset_for_finds_by_label_and_misses_unknown() {
    let openai = preset_for("OpenAI").expect("OpenAI preset");
    assert_eq!(openai.api_url, "https://api.openai.com/v1/chat/completions");
    assert_eq!(openai.default_model, "gpt-4o-mini");
    assert!(preset_for("Nope").is_none());
}

#[test]
fn custom_preset_has_no_url_and_never_matches_an_empty_url_by_accident() {
    let custom = preset_for("Custom").expect("Custom preset");
    assert_eq!(custom.api_url, "");
    assert_eq!(preset_label_for_url(custom.api_url), "Custom");
}
