"""Primary action + one status line that always says what happened or what to do next."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk
from typing import Callable

HINT = "Enter to rewrite  ·  Shift+Enter for a new line"
_TONES = {"muted": "Muted.TLabel", "success": "Success.TLabel", "danger": "Danger.TLabel"}
_TICK_MS = 350


class ActionBar(ttk.Frame):
    def __init__(self, parent: tk.Misc, *, on_rewrite: Callable[[], None]):
        super().__init__(parent)
        self.button = ttk.Button(self, text="Rewrite", style="Accent.TButton", command=on_rewrite)
        self.button.pack(side="left")
        self._status = ttk.Label(self, style="Muted.TLabel")
        self._status.pack(side="left", padx=(14, 0))
        self._job: str | None = None
        self._dots = 0
        self.say(HINT)

    def say(self, text: str, tone: str = "muted") -> None:
        self._status.configure(text=text, style=_TONES[tone])

    def set_busy(self, busy: bool) -> None:
        if self._job is not None:
            self.after_cancel(self._job)
            self._job = None
        self.button.configure(state="disabled" if busy else "normal")
        if busy:
            self._dots = 0
            self._tick()

    def _tick(self) -> None:
        self.say("Rewriting" + "." * (self._dots % 4))
        self._dots += 1
        self._job = self.after(_TICK_MS, self._tick)
