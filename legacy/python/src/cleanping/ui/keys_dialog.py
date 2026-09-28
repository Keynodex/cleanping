"""Modal dialog: saved keys on the left, one form on the right."""

from __future__ import annotations

import tkinter as tk
from tkinter import messagebox, ttk
from typing import Callable

from ..application.credentials import CredentialService
from ..domain.errors import CleanpingError
from .keys_form import KeyForm
from .labels import LabelIndex
from .theme import Theme


class KeysDialog(tk.Toplevel):
    def __init__(self, parent: tk.Misc, theme: Theme, service: CredentialService,
                 on_change: Callable[[], None], *, select_id: int | None = None):
        super().__init__(parent)
        self._service, self._on_change = service, on_change
        self._index = LabelIndex()
        p = theme.palette
        self.title("API keys")
        self.configure(bg=p.bg)
        self.geometry("800x500")
        self.minsize(760, 480)
        self.transient(parent)
        self.grab_set()
        self._build(theme)
        self._reload(select_id)
        self.bind("<Escape>", lambda _e: self.destroy())
        self.bind("<Control-s>", lambda _e: self._save())

    def _build(self, theme: Theme) -> None:
        p = theme.palette
        frame = ttk.Frame(self, padding=16)
        frame.pack(fill="both", expand=True)
        frame.columnconfigure(1, weight=1)
        frame.rowconfigure(0, weight=1)

        side = ttk.Frame(frame)
        side.grid(row=0, column=0, sticky="ns", padx=(0, 20))
        ttk.Button(side, text="+ New key", command=self._new_key).pack(fill="x", pady=(0, 8))
        self.listbox = tk.Listbox(
            side, exportselection=False, width=18, relief="flat", borderwidth=0,
            highlightthickness=1, highlightbackground=p.border, highlightcolor=p.border,
            activestyle="none", bg=p.surface, fg=p.text, selectbackground=p.select,
            selectforeground=p.text, font=theme.fonts.ui,
        )
        self.listbox.pack(fill="both", expand=True)
        self.listbox.bind("<<ListboxSelect>>", self._load_selected)

        self.form = KeyForm(frame)
        self.form.grid(row=0, column=1, sticky="new")
        bottom = ttk.Frame(frame)
        bottom.grid(row=1, column=0, columnspan=2, sticky="ew", pady=(16, 0))
        ttk.Button(bottom, text="Save", style="Accent.TButton", command=self._save).pack(side="left")
        ttk.Button(bottom, text="Delete", style="Danger.TButton", command=self._delete).pack(
            side="left", padx=(8, 0))
        self.status = ttk.Label(bottom, style="Muted.TLabel")
        self.status.pack(side="left", padx=(14, 0))
        ttk.Button(bottom, text="Close", command=self.destroy).pack(side="right")

    def _reload(self, select_id: int | None = None) -> None:
        self.listbox.delete(0, "end")
        labels = self._index.load(self._service.list(), lambda c: c.name)
        for position, label in enumerate(labels):
            self.listbox.insert("end", label)
            if self._index.id_for(label) == select_id:
                self.listbox.selection_set(position)
        if self.listbox.curselection():
            self._load_selected()
        elif not labels:
            self.form.focus_key()

    def _selected_id(self) -> int | None:
        selection = self.listbox.curselection()
        return self._index.id_for(self.listbox.get(selection[0])) if selection else None

    def _say(self, text: str, tone: str = "muted") -> None:
        style = {"muted": "Muted.TLabel", "danger": "Danger.TLabel", "success": "Success.TLabel"}
        self.status.configure(text=text, style=style[tone])

    def _load_selected(self, _event=None) -> None:
        credential_id = self._selected_id()
        if credential_id is None:
            return
        try:
            self.form.load(self._service.get(credential_id))
        except CleanpingError as exc:
            self._say(str(exc), "danger")

    def _new_key(self) -> None:
        self.listbox.selection_clear(0, "end")
        self.form.blank()
        self.form.focus_key()
        self._say("Pick a provider, paste the key, Save.")

    def _save(self) -> None:
        try:
            saved = self._service.save(self.form.read())
        except CleanpingError as exc:
            self._say(str(exc), "danger")
            return
        self.form.clear_key()
        self._reload(saved.id)
        self._on_change()
        self._say(f"Saved “{saved.name}”.", "success")

    def _delete(self) -> None:
        credential_id = self._selected_id()
        if credential_id is None:
            self._say("Pick a key in the list first.", "danger")
            return
        try:
            credential = self._service.get(credential_id)
            if not messagebox.askyesno(
                "Delete key", f"Delete “{credential.name}” and its stored API key?", parent=self):
                return
            self._service.delete(credential_id)
        except CleanpingError as exc:
            self._say(str(exc), "danger")
            return
        self.form.blank()
        self._reload()
        self._on_change()
        self._say(f"Deleted “{credential.name}”.")
