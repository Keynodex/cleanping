"""Use case service: small typed facade over the app_state key/value table."""

from __future__ import annotations

from ..domain.errors import ValidationError
from ..domain.versions import VersionStack
from .ports import StateRepository

_DRAFT = "draft"
_RESULT = "result"
_SELECTED_CREDENTIAL = "selected_credential_id"
_GEOMETRY = "window_geometry"
_VERSIONS = "versions"
_THEME = "theme"
_THEMES = ("light", "dark")


class AppState:
    """What the window restores on next launch; one row per key in SQLite."""

    def __init__(self, state: StateRepository):
        self._state = state

    def load_draft(self) -> str:
        return self._state.get(_DRAFT) or ""

    def save_draft(self, text: str) -> None:
        self._state.set(_DRAFT, text)

    def load_result(self) -> str:
        return self._state.get(_RESULT) or ""

    def save_result(self, text: str) -> None:
        self._state.set(_RESULT, text)

    def selected_credential_id(self) -> int | None:
        raw = self._state.get(_SELECTED_CREDENTIAL)
        try:
            return int(raw) if raw else None
        except ValueError:
            return None

    def select_credential(self, credential_id: int | None) -> None:
        self._state.set(_SELECTED_CREDENTIAL, "" if credential_id is None else str(credential_id))

    def load_geometry(self) -> str | None:
        return self._state.get(_GEOMETRY)

    def save_geometry(self, geometry: str) -> None:
        self._state.set(_GEOMETRY, geometry)

    def load_theme(self) -> str | None:
        """"light", "dark", or None when the user never chose (follow the system)."""
        value = self._state.get(_THEME)
        return value if value in _THEMES else None

    def save_theme(self, theme: str) -> None:
        if theme not in _THEMES:
            raise ValidationError(f"Unknown theme: {theme!r}.")
        self._state.set(_THEME, theme)

    def load_versions(self) -> VersionStack:
        """The editor history; adopts the old draft/result pair the first time."""
        raw = self._state.get(_VERSIONS)
        if raw:
            return VersionStack.loads(raw)
        legacy = [t for t in (self.load_draft(), self.load_result()) if t.strip()]
        return VersionStack(legacy or None)

    def save_versions(self, stack: VersionStack) -> None:
        self._state.set(_VERSIONS, stack.dumps())
        for legacy_key in (_DRAFT, _RESULT):  # adopted into the stack; leave no stale copy
            if self._state.get(legacy_key):
                self._state.set(legacy_key, "")
