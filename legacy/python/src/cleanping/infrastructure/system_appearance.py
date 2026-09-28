"""Ask the desktop whether it prefers a dark colour scheme. Fails soft to light."""

from __future__ import annotations

import subprocess

_GSETTINGS = ("gsettings", "get", "org.gnome.desktop.interface", "color-scheme")


def prefers_dark() -> bool:
    try:
        out = subprocess.run(
            _GSETTINGS, capture_output=True, text=True, timeout=1.5, check=False
        ).stdout
    except (OSError, subprocess.SubprocessError):
        return False
    return "dark" in out.lower()
