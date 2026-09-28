"""Main window: composes the panes and routes events. Decisions live in the use cases."""

from __future__ import annotations

import tkinter as tk

from ..application.app_state import AppState
from ..application.credentials import CredentialService
from ..application.polisher import PolishText
from ..application.prompts import PromptService
from ..domain.errors import CleanpingError
from ..domain.models import Credential
from .keys_dialog import KeysDialog
from .history import EditorHistory
from .polish_runner import PolishOutcome, PolishRunner
from .prompt_dialog import PromptDialog
from .theme import Theme
from .workspace import Handlers, Workspace

_GEOMETRY_FALLBACK = "760x620"


class CleanpingWindow:
    def __init__(self, root: tk.Tk, *, polish: PolishText, credentials: CredentialService,
                 prompts: PromptService, state: AppState, theme: Theme):
        self._root, self._credentials, self._prompts = root, credentials, prompts
        self._state, self._theme = state, theme
        self._build()
        self._restore()
        self._runner = PolishRunner(root, polish, self._present)
        root.protocol("WM_DELETE_WINDOW", self._on_close)
        if not self._credentials.list():
            root.after(250, self._open_keys)

    def _build(self) -> None:
        self._root.title("Cleanping")
        self._root.minsize(520, 420)
        handlers = Handlers(
            select=self._state.select_credential, pin=self._apply_pin, keys=self._open_keys,
            prompt=self._open_prompt, theme=self._switch_theme, rewrite=self._submit,
            text_changed=lambda: self._history.text_changed(),
            text_left=lambda: self._history.save(), clear=lambda: self._history.clear(),
            copy=self._copy, back=lambda: self._history.back(),
            forward=lambda: self._history.forward(),
            toggle_original=lambda: self._history.toggle_original(),
        )
        space = Workspace(self._root, self._theme, handlers)
        space.pack(fill="both", expand=True)
        self._toolbar, self._editor, self._actions = space.toolbar, space.editor, space.actions
        self._history = EditorHistory(self._root, self._editor, self._actions, self._state)

    def _restore(self) -> None:
        self._root.geometry(self._state.load_geometry() or _GEOMETRY_FALLBACK)
        self._apply_pin()
        self._history.show()
        self._editor.focus()
        self._refresh_credentials()

    def _apply_pin(self) -> None:
        self._root.wm_attributes("-topmost", self._toolbar.pinned)

    def _refresh_credentials(self) -> None:
        selected = self._toolbar.refresh(self._credentials.list(), self._state.selected_credential_id())
        self._state.select_credential(selected)

    def _selected_credential(self) -> Credential | None:
        credential_id = self._toolbar.selected_id()
        try:
            return None if credential_id is None else self._credentials.get(credential_id)
        except CleanpingError:
            return None

    # ----------------------------------------------------------- rewriting

    def _submit(self) -> None:
        source = self._editor.get().strip()
        if not source:
            self._actions.say("Nothing to rewrite yet. Type or paste some text first.", "danger")
            return
        credential = self._selected_credential()
        if credential is None:
            self._actions.say("Add an API key first: Settings ▾ → API keys… (Alt+K).", "danger")
            return
        self._history.save()
        if self._runner.start(source, self._prompts.current(), credential):
            self._editor.set_editable(False)
            self._actions.set_busy(True)

    def _present(self, outcome: PolishOutcome) -> None:
        self._actions.set_busy(False)
        self._editor.set_editable(True)
        result = outcome.result
        if result is None or not result.ok:
            self._actions.say(outcome.error or (result.error_message if result else "Unknown error"), "danger")
            return
        self._history.push(result.output_text or "")
        self._copy(f"Copied · {result.run.duration_ms / 1000:.1f} s · Alt+O shows original")

    def _copy(self, message: str = "Copied to clipboard.") -> None:
        value = self._editor.get().strip()
        if not value:
            self._actions.say("Nothing to copy yet.", "danger")
            return
        self._root.clipboard_clear()
        self._root.clipboard_append(value)
        self._root.update_idletasks()
        self._actions.say(message, "success")

    # -------------------------------------------------------------- window

    def _switch_theme(self) -> None:
        self._theme.toggle()
        self._state.save_theme("dark" if self._theme.dark else "light")

    def _open_keys(self) -> None:
        KeysDialog(self._root, self._theme, self._credentials, self._refresh_credentials,
                   select_id=self._toolbar.selected_id())

    def _open_prompt(self) -> None:
        PromptDialog(self._root, self._theme, self._prompts)

    def _on_close(self) -> None:
        self._history.save()
        self._state.save_geometry(self._root.winfo_geometry())
        self._root.destroy()
