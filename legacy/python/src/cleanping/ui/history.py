"""Binds the editor to its VersionStack: typing, stepping, rewrites and autosave."""

from __future__ import annotations

import tkinter as tk
from typing import Callable

from ..application.app_state import AppState
from .action_bar import ActionBar
from .labels import TOGGLE_LABEL
from .text_pane import TextPane

_SAVE_DELAY_MS = 400


class EditorHistory:
    def __init__(self, root: tk.Misc, editor: TextPane, actions: ActionBar, state: AppState):
        self._root, self._editor, self._actions, self._state = root, editor, actions, state
        self._stack = state.load_versions()
        self._toggle = editor.button(TOGGLE_LABEL)
        self._save_job: str | None = None

    def show(self) -> None:
        """Put the current version in the editor and refresh the header."""
        self._editor.set(self._stack.current)
        self._label()

    def text_changed(self) -> None:
        """The user typed: fold it into the stack (forks when an old version is shown)."""
        text = self._editor.get()
        if text == self._stack.current:
            return
        self._stack.edit(text)
        self._label()
        if self._save_job is not None:
            self._root.after_cancel(self._save_job)
        self._save_job = self._root.after(_SAVE_DELAY_MS, self.save)

    def save(self) -> None:
        self._save_job = None
        self.text_changed()
        self._state.save_versions(self._stack)

    def back(self) -> None:
        self._step(self._stack.back)

    def forward(self) -> None:
        self._step(self._stack.forward)

    def toggle_original(self) -> None:
        self._step(self._stack.toggle_original)

    def _step(self, move: Callable[[], bool]) -> None:
        self.text_changed()
        if not move():
            self._actions.say("No other version to show.")
            return
        self.show()
        self.save()
        stack = self._stack
        self._actions.say(
            "Showing the original." if stack.index == 0
            else f"Showing version {stack.index + 1} of {stack.count}."
        )

    def push(self, text: str) -> None:
        """A rewrite arrived: keep what was there, show the new text."""
        self.text_changed()
        self._stack.push(text)
        self.show()
        self.save()

    def clear(self) -> None:
        self.text_changed()
        if self._stack.push(""):
            self.show()
            self.save()
            self._actions.say("Cleared. Alt+Z brings the previous text back.")

    def _label(self) -> None:
        stack = self._stack
        self._editor.set_suffix("" if stack.count == 1 else f"{stack.index + 1} of {stack.count}")
        self._toggle.configure(
            text="Latest" if stack.index == 0 and stack.count > 1 else TOGGLE_LABEL,
            state="normal" if stack.count > 1 else "disabled",
        )
