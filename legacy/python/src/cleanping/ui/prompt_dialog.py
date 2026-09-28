"""Modal dialog: edit the system prompt sent with every rewrite request."""

from __future__ import annotations

import tkinter as tk
from tkinter import messagebox, ttk

from ..application.prompts import DEFAULT_INSTRUCTIONS, PromptService
from ..domain.errors import CleanpingError
from .text_pane import TextPane
from .theme import Theme


class PromptDialog(tk.Toplevel):
    """One editor; Reset restores the shipped default, Save applies it."""

    def __init__(self, parent: tk.Misc, theme: Theme, prompts: PromptService):
        super().__init__(parent)
        self._prompts = prompts
        self._dirty = False
        self.title("System prompt")
        self.configure(bg=theme.palette.bg)
        self.geometry("680x440")
        self.minsize(520, 340)
        self.transient(parent)
        self.grab_set()
        self._build(theme)
        self.protocol("WM_DELETE_WINDOW", self._close)
        self.bind("<Escape>", lambda _e: self._close())
        self.bind("<Control-s>", lambda _e: self._save())

    def _build(self, theme: Theme) -> None:
        frame = ttk.Frame(self, padding=16)
        frame.pack(fill="both", expand=True)
        frame.columnconfigure(0, weight=1)
        frame.rowconfigure(1, weight=1)
        ttk.Label(frame, text="Sent with every rewrite. Tell the model how to edit your draft.",
                  style="Muted.TLabel").grid(row=0, column=0, sticky="w", pady=(0, 8))
        self.pane = TextPane(
            frame, theme, title="Instructions", placeholder="Write the editing instructions…",
            actions=[("Reset to default", self._reset)], on_change=self._mark_dirty,
        )
        self.pane.grid(row=1, column=0, sticky="nsew")
        self.pane.set(self._prompts.current())
        self.pane.focus()
        bottom = ttk.Frame(frame)
        bottom.grid(row=2, column=0, sticky="ew", pady=(14, 0))
        ttk.Button(bottom, text="Save", style="Accent.TButton", command=self._save).pack(side="left")
        self.status = ttk.Label(bottom, style="Muted.TLabel", text="Saved")
        self.status.pack(side="left", padx=(14, 0))
        ttk.Button(bottom, text="Close", command=self._close).pack(side="right")

    def _mark_dirty(self) -> None:
        self._dirty = True
        self.status.configure(text="Unsaved changes (Ctrl+S to save)", style="Muted.TLabel")

    def _save(self) -> None:
        try:
            self._prompts.save(self.pane.get())
        except CleanpingError as exc:
            self.status.configure(text=str(exc), style="Danger.TLabel")
            return
        self._dirty = False
        self.status.configure(text="Saved. Used on the next rewrite.", style="Success.TLabel")

    def _reset(self) -> None:
        self.pane.set(DEFAULT_INSTRUCTIONS)
        self.status.configure(text="Default loaded. Save to apply.", style="Muted.TLabel")

    def _close(self) -> None:
        if self._dirty and messagebox.askyesno(
                "Unsaved changes", "Save the prompt before closing?", parent=self):
            self._save()
            if self._dirty:
                return  # save failed; keep the window open
        self.destroy()
