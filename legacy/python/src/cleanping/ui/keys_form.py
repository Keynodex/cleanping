"""The credential form: provider first, then key; preset fills URL, model and name."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk

from ..domain.models import Credential, CredentialInput
from ..domain.providers import PRESET_LABELS, preset_for, preset_label_for_url

HINT_NEW = "Saved locally, never shown again."
HINT_EDIT = "Blank keeps the saved key."


class KeyForm(ttk.Frame):
    def __init__(self, parent: tk.Misc):
        super().__init__(parent)
        self.columnconfigure((0, 1), weight=1, uniform="cols")
        self.provider = tk.StringVar(value="Custom")
        self.name, self.model, self.url = tk.StringVar(), tk.StringVar(), tk.StringVar()

        box = self._field("Provider", 0, 0, 2, ttk.Combobox(
            self, state="readonly", values=list(PRESET_LABELS), textvariable=self.provider))
        box.bind("<<ComboboxSelected>>", self._apply_preset)
        self.key_entry = self._field("API key", 2, 0, 2, ttk.Entry(self, show="•"))
        self.hint = ttk.Label(self, style="Muted.TLabel", text=HINT_NEW)
        self.hint.grid(row=4, column=0, columnspan=2, sticky="w", pady=(3, 0))
        self._field("Model", 5, 0, 1, ttk.Entry(self, textvariable=self.model))
        self._field("Name", 5, 1, 1, ttk.Entry(self, textvariable=self.name))
        self._field("API URL", 7, 0, 2, ttk.Entry(self, textvariable=self.url))

    def _field(self, label: str, row: int, column: int, span: int, widget):
        ttk.Label(self, text=label, style="Field.TLabel").grid(
            row=row, column=column, columnspan=span, sticky="w", pady=(8 if row else 0, 3))
        widget.grid(row=row + 1, column=column, columnspan=span, sticky="ew",
                    padx=(0, 8) if column == 0 and span == 1 else 0)
        return widget

    def blank(self) -> None:
        self._set("Custom", "", "", "")
        self.hint.configure(text=HINT_NEW)

    def load(self, credential: Credential) -> None:
        self._set(preset_label_for_url(credential.api_url), credential.name,
                  credential.api_url, credential.model)
        self.hint.configure(text=HINT_EDIT)

    def read(self) -> CredentialInput:
        return CredentialInput(name=self.name.get(), api_url=self.url.get(),
                               model=self.model.get(), api_key=self.key_entry.get())

    def clear_key(self) -> None:
        self.key_entry.delete(0, "end")

    def focus_key(self) -> None:
        self.key_entry.focus_set()

    def _set(self, provider: str, name: str, url: str, model: str) -> None:
        self.provider.set(provider)
        self.name.set(name)
        self.url.set(url)
        self.model.set(model)
        self.clear_key()

    def _apply_preset(self, _event=None) -> None:
        preset = preset_for(self.provider.get())
        if preset is None:
            return
        self.url.set(preset.api_url)
        self.model.set(preset.default_model)
        if not self.name.get().strip() or self.name.get() in PRESET_LABELS:
            self.name.set(preset.label)
        self.key_entry.focus_set()
