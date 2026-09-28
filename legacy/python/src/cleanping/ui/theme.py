"""Palettes, fonts and the runtime light/dark switch. Widgets subscribe to restyle."""

from __future__ import annotations

import tkinter as tk
import tkinter.font as tkfont
from dataclasses import dataclass
from tkinter import ttk
from typing import Callable

from .styles import configure_styles


@dataclass(frozen=True)
class Palette:
    bg: str
    surface: str
    border: str
    text: str
    muted: str
    accent: str
    accent_hover: str
    accent_text: str
    success: str
    danger: str
    select: str


LIGHT = Palette(
    bg="#F6F6F7", surface="#FFFFFF", border="#E3E3E7", text="#18181B",
    muted="#71717A", accent="#4F46E5", accent_hover="#4338CA", accent_text="#FFFFFF",
    success="#15803D", danger="#DC2626", select="#DCDCFB",
)
DARK = Palette(
    bg="#121214", surface="#1B1B1F", border="#2C2C32", text="#ECECEF",
    muted="#8E8E98", accent="#8B8BF5", accent_hover="#A0A0F8", accent_text="#0F0F12",
    success="#4ADE80", danger="#F87171", select="#33336B",
)

_FAMILIES = ("Inter", "Segoe UI", "SF Pro Text", "Noto Sans", "DejaVu Sans")


@dataclass(frozen=True)
class Fonts:
    ui: tkfont.Font
    small: tkfont.Font
    bold: tkfont.Font
    text: tkfont.Font


class Theme:
    """Owns the ttk style; `toggle()` flips light/dark and notifies subscribers."""

    def __init__(self, root: tk.Tk, *, dark: bool):
        self._root = root
        self.dark = dark
        self._listeners: list[Callable[[Palette], None]] = []
        available = set(tkfont.families(root))
        family = next((f for f in _FAMILIES if f in available), "TkDefaultFont")

        def font(size: int, weight: str = "normal") -> tkfont.Font:
            return tkfont.Font(root=root, family=family, size=size, weight=weight)

        self.fonts = Fonts(ui=font(10), small=font(9), bold=font(10, "bold"), text=font(11))
        self._style = ttk.Style(root)
        self._style.theme_use("clam")
        self._apply()

    @property
    def palette(self) -> Palette:
        return DARK if self.dark else LIGHT

    def toggle(self) -> None:
        self.dark = not self.dark
        self._apply()
        for listener in list(self._listeners):
            listener(self.palette)

    def subscribe(self, listener: Callable[[Palette], None]) -> Callable[[], None]:
        self._listeners.append(listener)

        def unsubscribe() -> None:
            if listener in self._listeners:
                self._listeners.remove(listener)

        return unsubscribe

    def _apply(self) -> None:
        self._root.configure(bg=self.palette.bg)
        configure_styles(self._style, self.palette, self.fonts)
