"""Provider preset lookup: URL -> dropdown label."""

from cleanping.domain.providers import PROVIDER_PRESETS, preset_label_for_url


def test_known_url_maps_to_its_preset_label():
    preset = PROVIDER_PRESETS[0]
    assert preset_label_for_url(preset.api_url) == preset.label


def test_unknown_url_is_custom():
    assert preset_label_for_url("https://example.test/v1/chat/completions") == "Custom"


def test_empty_url_is_custom():
    assert preset_label_for_url("") == "Custom"
