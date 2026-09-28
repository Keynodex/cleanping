"""Builds the real window against scratch data and drives its handlers. Skips without a display."""

import tkinter as tk

import pytest

from cleanping import __main__ as entry
from cleanping.application.polisher import PolishResult
from cleanping.domain.models import Run
from cleanping.ui.polish_runner import PolishOutcome


@pytest.fixture
def window(tmp_path, monkeypatch):
    monkeypatch.setenv("XDG_CONFIG_HOME", str(tmp_path / "config"))
    monkeypatch.setenv("XDG_DATA_HOME", str(tmp_path / "data"))
    monkeypatch.setenv("LLM_API_KEY", "not-a-real-key")
    monkeypatch.setenv("LLM_MODEL", "test-model")
    monkeypatch.setenv("LLM_API_URL", "https://api.example.test/v1/chat/completions")
    try:
        root = tk.Tk()
    except tk.TclError:
        pytest.skip("no display available")
    root.withdraw()
    win = entry.build_window(root)
    yield win
    root.destroy()


def _ok_outcome(text: str) -> PolishOutcome:
    run = Run(input_text="x", output_text=text, status="ok", error_message=None,
              duration_ms=1200, credential_name="Environment", model="test-model", prompt_text="p")
    return PolishOutcome(result=PolishResult(output_text=text, error_message=None, run=run))


def _status(window) -> str:
    return window._actions._status.cget("text")


def _rewrite(window, source: str, output: str) -> None:
    window._editor.set(source)
    window._history.text_changed()
    window._present(_ok_outcome(output))


def test_seeded_credential_is_selected(window):
    assert window._toolbar.selected_id() is not None


def test_empty_text_is_refused_with_a_reason(window):
    window._submit()
    assert "Nothing to rewrite" in _status(window)


def test_rewrite_replaces_the_text_copies_it_and_keeps_the_original(window):
    _rewrite(window, "rough draft", "Polished text.")
    assert window._editor.get() == "Polished text."
    assert window._root.clipboard_get() == "Polished text."
    window._history.toggle_original()
    assert window._editor.get() == "rough draft"
    window._history.toggle_original()
    assert window._editor.get() == "Polished text."


def test_error_outcome_is_shown_and_keeps_the_text(window):
    window._editor.set("keep me")
    window._present(PolishOutcome(error="No API key saved."))
    assert "No API key saved." in _status(window)
    assert window._editor.get() == "keep me"
    assert str(window._editor.text.cget("state")) == "normal"


def test_clear_is_undoable_and_does_not_lose_the_rewrite(window):
    _rewrite(window, "rough", "Polished")
    window._history.clear()
    assert window._editor.get() == ""
    window._history.back()
    assert window._editor.get() == "Polished"


def test_editing_an_older_version_forks_instead_of_overwriting(window):
    _rewrite(window, "rough", "Polished")
    window._history.back()
    window._editor.set("rough, edited")
    window._history.text_changed()
    window._history.back()
    assert window._editor.get() == "Polished"
    window._history.toggle_original()
    assert window._editor.get() == "rough"


def test_versions_survive_a_restart(window):
    _rewrite(window, "rough", "Polished")
    restored = window._state.load_versions()
    assert restored.versions[-2:] == ["rough", "Polished"]


def test_theme_switch_is_persisted(window):
    before = window._theme.dark
    window._switch_theme()
    assert window._theme.dark is (not before)
    assert window._state.load_theme() == ("dark" if window._theme.dark else "light")
