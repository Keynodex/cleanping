"""Top strip: model picker on the left; pin and settings menu on the right."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk
from typing import Callable

from ..domain.models import Credential
from .labels import LabelIndex
from .theme import Palette, Theme


class Toolbar(ttk.Frame):
    def __init__(
        self,
        parent: tk.Misc,
        theme: Theme,
        *,
        on_select: Callable[[int | None], None],
        on_pin: Callable[[], None],
        menu_items: list[tuple[str, str, Callable[[], None]]],
    ):
        super().__init__(parent)
        self._on_select = on_select
        self._on_pin = on_pin
        self._index = LabelIndex()
        self.pinned = True

        self.combo = ttk.Combobox(self, state="readonly", width=24)
        self.combo.pack(side="left")
        self.combo.bind("<<ComboboxSelected>>", lambda _e: on_select(self.selected_id()))

        self._menu = tk.Menu(self, tearoff=0, relief="flat", borderwidth=1)
        for label, accelerator, command in menu_items:
            self._menu.add_command(label=label, accelerator=accelerator, command=command)
        settings = ttk.Button(self, text="Settings ▾", style="Ghost.TButton", command=self._popup)
        settings.pack(side="right")
        self._settings = settings
        self._pin = ttk.Button(self, style="GhostOn.TButton", command=self._toggle_pin)
        self._pin.pack(side="right", padx=(0, 4))
        self._show_pin()
        self._restyle(theme.palette)
        unsubscribe = theme.subscribe(self._restyle)
        self.bind("<Destroy>", lambda _e: unsubscribe(), add="+")

    def refresh(self, credentials: list[Credential], selected_id: int | None) -> int | None:
        """Reload choices; returns the id now selected (None when there are none)."""
        labels = self._index.load(credentials)
        self.combo.configure(values=labels)
        label = self._index.label_for(selected_id) or (labels[0] if labels else "")
        self.combo.set(label)
        return self._index.id_for(label)

    def selected_id(self) -> int | None:
        return self._index.id_for(self.combo.get())

    def popup_settings(self) -> None:
        self._popup()

    def _popup(self) -> None:
        x = self._settings.winfo_rootx()
        y = self._settings.winfo_rooty() + self._settings.winfo_height()
        self._menu.tk_popup(x, y)

    def _toggle_pin(self) -> None:
        self.pinned = not self.pinned
        self._show_pin()
        self._on_pin()

    def _show_pin(self) -> None:
        self._pin.configure(
            text="Pinned" if self.pinned else "Pin",
            style="GhostOn.TButton" if self.pinned else "Ghost.TButton",
        )

    def _restyle(self, p: Palette) -> None:
        self._menu.configure(
            bg=p.surface, fg=p.text, activebackground=p.select, activeforeground=p.text,
            disabledforeground=p.muted,
        )
        try:  # the combobox drop-down list is a separate Tk window
            popdown = self.tk.eval(f"ttk::combobox::PopdownWindow {self.combo}")
            self.tk.call(
                f"{popdown}.f.l", "configure", "-background", p.surface, "-foreground", p.text,
                "-selectbackground", p.select, "-selectforeground", p.text, "-borderwidth", 0,
            )
        except tk.TclError:
            pass
