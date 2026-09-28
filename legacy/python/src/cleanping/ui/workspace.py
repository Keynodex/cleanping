"""The window's layout: toolbar, one editor, action bar. Behavior comes from handlers."""

from __future__ import annotations

from dataclasses import dataclass
from tkinter import ttk
from typing import Callable

from .action_bar import ActionBar
from .labels import TOGGLE_LABEL
from .text_pane import TextPane
from .theme import Theme
from .toolbar import Toolbar

_SHIFT_MASK = 0x1
Handler = Callable[[], None]


@dataclass(frozen=True)
class Handlers:
    select: Callable[[int | None], None]
    pin: Handler
    keys: Handler
    prompt: Handler
    theme: Handler
    rewrite: Handler
    text_changed: Handler
    text_left: Handler
    clear: Handler
    copy: Handler
    back: Handler
    forward: Handler
    toggle_original: Handler


class Workspace(ttk.Frame):
    """Owns widget assembly only; every action is delegated to a handler."""

    def __init__(self, parent, theme: Theme, h: Handlers):
        super().__init__(parent, padding=(16, 12, 16, 16))
        self._rewrite = h.rewrite
        self.columnconfigure(0, weight=1)
        self.rowconfigure(1, weight=1)

        self.toolbar = Toolbar(
            self, theme, on_select=h.select, on_pin=h.pin,
            menu_items=[("API keys…", "Alt+K", h.keys), ("System prompt…", "Alt+P", h.prompt),
                        ("Switch light / dark", "Alt+T", h.theme)],
        )
        self.toolbar.grid(row=0, column=0, sticky="ew", pady=(0, 10))
        self.editor = TextPane(
            self, theme, title="Text", placeholder="Paste or type your rough prompt, then press Enter…",
            actions=[("‹", h.back), ("›", h.forward), (TOGGLE_LABEL, h.toggle_original),
                     ("Copy", h.copy), ("Clear", h.clear)],
            on_change=h.text_changed,
        )
        self.editor.grid(row=1, column=0, sticky="nsew")
        self.actions = ActionBar(self, on_rewrite=h.rewrite)
        self.actions.grid(row=2, column=0, sticky="ew", pady=(12, 0))

        self.editor.text.bind("<Return>", self._on_enter)
        self.editor.text.bind("<FocusOut>", lambda _e: h.text_left())
        self.editor.text.bind("<Alt-c>", lambda _e: h.clear() or "break")
        self._bind_shortcuts(h)

    def _bind_shortcuts(self, h: Handlers) -> None:
        """Alt+key, not Ctrl+key: Tk's Text already owns Ctrl+K/P/T/O/Z-style editing."""
        window = self.winfo_toplevel()
        for key, action in (("k", h.keys), ("p", h.prompt), ("t", h.theme),
                            ("o", h.toggle_original), ("z", h.back), ("Z", h.forward)):
            window.bind(f"<Alt-{key}>", lambda _e, a=action: a() or "break")
        window.bind("<Control-Return>", lambda _e: h.rewrite() or "break")
        window.bind("<Control-Shift-C>", lambda _e: h.copy() or "break")

    def _on_enter(self, event):
        if event.state & _SHIFT_MASK:
            return None  # Shift+Enter inserts a newline.
        self._rewrite()
        return "break"
