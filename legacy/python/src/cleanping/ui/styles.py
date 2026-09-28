"""ttk style definitions for the flat, minimal look. Pure configuration, no widgets."""

from __future__ import annotations

from tkinter import ttk


def configure_styles(style: ttk.Style, p, fonts) -> None:
    style.configure(
        ".", background=p.bg, foreground=p.text, font=fonts.ui, bordercolor=p.border,
        lightcolor=p.bg, darkcolor=p.bg, troughcolor=p.bg, focuscolor=p.bg,
        selectbackground=p.select, selectforeground=p.text,
    )
    for name, bg in (("TFrame", p.bg), ("Surface.TFrame", p.surface),
                     ("Ring.TFrame", p.border), ("RingFocus.TFrame", p.accent)):
        style.configure(name, background=bg)

    style.configure("TLabel", background=p.bg, foreground=p.text)
    style.configure("Muted.TLabel", foreground=p.muted, font=fonts.small)
    style.configure("Success.TLabel", foreground=p.success, font=fonts.small)
    style.configure("Danger.TLabel", foreground=p.danger, font=fonts.small)
    style.configure("Field.TLabel", foreground=p.muted, font=fonts.small)
    style.configure("Surface.TLabel", background=p.surface, foreground=p.muted, font=fonts.small)
    style.configure("PaneTitle.TLabel", background=p.surface, foreground=p.text, font=fonts.bold)
    style.configure("Placeholder.TLabel", background=p.surface, foreground=p.muted, font=fonts.text)

    style.configure("TButton", width=0, padding=(12, 6), relief="flat", borderwidth=1,
                    background=p.surface, foreground=p.text, bordercolor=p.border)
    style.map("TButton", background=[("active", p.border)],
              foreground=[("disabled", p.muted)])
    style.configure("Accent.TButton", padding=(20, 8), background=p.accent, font=fonts.bold,
                    foreground=p.accent_text, bordercolor=p.accent)
    style.map("Accent.TButton",
              background=[("disabled", p.border), ("active", p.accent_hover)],
              bordercolor=[("disabled", p.border), ("active", p.accent_hover)],
              foreground=[("disabled", p.muted)])
    ghost = dict(padding=(8, 4), borderwidth=0, background=p.bg, foreground=p.muted,
                 font=fonts.small)
    style.configure("Ghost.TButton", **ghost)
    style.configure("GhostOn.TButton", **{**ghost, "foreground": p.accent})
    style.configure("Danger.TButton", **{**ghost, "foreground": p.danger})
    for name in ("Ghost.TButton", "GhostOn.TButton", "Danger.TButton"):
        style.map(name, background=[("active", p.border)])
    style.configure("PaneAction.TButton", **{**ghost, "background": p.surface, "padding": (6, 2)})
    style.map("PaneAction.TButton", background=[("active", p.surface)],
              foreground=[("active", p.text)])

    field = dict(fieldbackground=p.surface, background=p.surface, foreground=p.text,
                 bordercolor=p.border, lightcolor=p.surface, darkcolor=p.surface,
                 padding=6, arrowcolor=p.muted, insertcolor=p.text)
    style.configure("TCombobox", **field)
    style.configure("TEntry", **field)
    focus_ring = [("focus", p.accent)]
    for name in ("TCombobox", "TEntry"):
        style.map(name, bordercolor=focus_ring, lightcolor=focus_ring, darkcolor=focus_ring)
    style.map("TCombobox", fieldbackground=[("readonly", p.surface)],
              foreground=[("readonly", p.text)],
              selectbackground=[("readonly", p.surface)],
              selectforeground=[("readonly", p.text)])

    style.layout("Slim.Vertical.TScrollbar", [("Vertical.Scrollbar.trough", {
        "sticky": "ns", "children": [("Vertical.Scrollbar.thumb", {"expand": "1", "sticky": "nswe"})]})])
    style.configure("Slim.Vertical.TScrollbar", background=p.border, troughcolor=p.surface,
                    bordercolor=p.surface, lightcolor=p.border, darkcolor=p.border, width=7)
    style.map("Slim.Vertical.TScrollbar", background=[("active", p.muted)])
