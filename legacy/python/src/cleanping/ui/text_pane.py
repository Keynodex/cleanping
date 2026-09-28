"""A flat text area: title, live word count, small actions, placeholder, focus ring."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk
from typing import Callable, Sequence

from .theme import Palette, Theme

_PAD_X, _PAD_Y = 14, 8


class TextPane(ttk.Frame):
    """One-purpose editor pane. `text` is exposed so callers can bind keys."""

    def __init__(
        self,
        parent: tk.Misc,
        theme: Theme,
        *,
        title: str,
        placeholder: str,
        actions: Sequence[tuple[str, Callable[[], None]]] = (),
        on_change: Callable[[], None] | None = None,
    ):
        super().__init__(parent, style="Ring.TFrame", padding=1)
        self._on_change = on_change
        self._suffix = ""
        self._buttons: dict[str, ttk.Button] = {}
        body = ttk.Frame(self, style="Surface.TFrame")
        body.pack(fill="both", expand=True)
        body.columnconfigure(0, weight=1)
        body.rowconfigure(1, weight=1)

        header = ttk.Frame(body, style="Surface.TFrame", padding=(_PAD_X, 8, 8, 0))
        header.grid(row=0, column=0, columnspan=2, sticky="ew")
        ttk.Label(header, text=title, style="PaneTitle.TLabel").pack(side="left")
        self._count = ttk.Label(header, style="Surface.TLabel")
        self._count.pack(side="left", padx=(10, 0))
        for label, command in reversed(actions):
            button = ttk.Button(header, text=label, style="PaneAction.TButton", command=command)
            button.pack(side="right")
            self._buttons[label] = button

        self.text = tk.Text(
            body, wrap="word", undo=True, relief="flat", borderwidth=0, highlightthickness=0,
            padx=_PAD_X, pady=_PAD_Y, spacing2=3, font=theme.fonts.text,
        )
        self.text.grid(row=1, column=0, sticky="nsew")
        scroll = ttk.Scrollbar(body, style="Slim.Vertical.TScrollbar", command=self.text.yview)
        scroll.grid(row=1, column=1, sticky="ns")
        self.text.configure(yscrollcommand=scroll.set)

        self._hint = ttk.Label(body, text=placeholder, style="Placeholder.TLabel")
        self._hint.bind("<Button-1>", lambda _e: self.text.focus_set())
        self.text.bind("<<Modified>>", self._modified)
        self.text.bind("<FocusIn>", lambda _e: self.configure(style="RingFocus.TFrame"))
        self.text.bind("<FocusOut>", lambda _e: self.configure(style="Ring.TFrame"))
        self._restyle(theme.palette)
        unsubscribe = theme.subscribe(self._restyle)
        self.bind("<Destroy>", lambda _e: unsubscribe(), add="+")
        self._refresh()

    # ---------------------------------------------------------------- API

    def get(self) -> str:
        return self.text.get("1.0", "end-1c")

    def set(self, value: str) -> None:
        self.text.delete("1.0", "end")
        self.text.insert("1.0", value)
        self.text.edit_reset()

    def clear(self) -> None:
        self.text.delete("1.0", "end")

    def focus(self) -> None:
        self.text.focus_set()

    def button(self, label: str) -> ttk.Button:
        return self._buttons[label]

    def set_suffix(self, suffix: str) -> None:
        """Extra header info after the word count, e.g. the version position."""
        self._suffix = suffix
        self._refresh()

    def set_editable(self, editable: bool) -> None:
        self.text.configure(state="normal" if editable else "disabled")

    # ------------------------------------------------------------ internals

    def _restyle(self, p: Palette) -> None:
        self.text.configure(
            bg=p.surface, fg=p.text, insertbackground=p.text, selectbackground=p.select,
            selectforeground=p.text, inactiveselectbackground=p.select,
        )

    def _modified(self, _event=None) -> None:
        if not self.text.edit_modified():
            return
        self.text.edit_modified(False)
        self._refresh()
        if self._on_change:
            self._on_change()

    def _refresh(self) -> None:
        words = len(self.get().split())
        count = f"{words} word{'s' if words != 1 else ''}" if words else ""
        self._count.configure(text=" · ".join(part for part in (count, self._suffix) if part))
        if words or self.get():
            self._hint.place_forget()
        else:
            self._hint.place(in_=self.text, x=_PAD_X, y=_PAD_Y)
