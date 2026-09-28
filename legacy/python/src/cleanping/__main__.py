"""Composition root: wire infrastructure into use cases into the window."""

from __future__ import annotations

import os
import tkinter as tk

from .application.app_state import AppState
from .application.credentials import CredentialService
from .application.polisher import PolishText
from .application.prompts import PromptService
from .domain.errors import CleanpingError
from .domain.models import CredentialInput
from .infrastructure.http_rewriter import OpenAIRewriter
from .infrastructure.paths import database_path, secrets_path
from .infrastructure.secrets_file import JsonSecretStore
from .infrastructure.system_appearance import prefers_dark
from .infrastructure.sqlite_db import Database
from .infrastructure.sqlite_repositories import (
    SqliteCredentialRepository,
    SqlitePromptRepository,
    SqliteRunRepository,
    SqliteStateRepository,
)
from .ui.main_window import CleanpingWindow
from .ui.theme import Theme

_DEFAULT_API_URL = "https://api.openai.com/v1/chat/completions"


def _seed_from_environment(credentials: CredentialService) -> None:
    """First run: adopt legacy LLM_* env vars as an editable credential."""
    if credentials.list():
        return
    api_key = os.environ.get("LLM_API_KEY", "").strip()
    model = os.environ.get("LLM_MODEL", "").strip()
    if not api_key or not model:
        return
    api_url = os.environ.get("LLM_API_URL", _DEFAULT_API_URL).strip()
    try:
        credentials.save(
            CredentialInput(
                name="Environment", api_url=api_url, model=model, api_key=api_key
            )
        )
    except CleanpingError:
        return  # a bad env URL must not block startup


def build_window(root: tk.Tk) -> CleanpingWindow:
    database = Database(database_path())
    database.migrate()
    secrets = JsonSecretStore(secrets_path())

    credentials = CredentialService(SqliteCredentialRepository(database), secrets)
    _seed_from_environment(credentials)

    state = AppState(SqliteStateRepository(database))
    saved = state.load_theme()
    theme = Theme(root, dark=(saved == "dark") if saved else prefers_dark())

    return CleanpingWindow(
        root,
        polish=PolishText(OpenAIRewriter(), SqliteRunRepository(database), secrets),
        credentials=credentials,
        prompts=PromptService(SqlitePromptRepository(database)),
        state=state,
        theme=theme,
    )


def main() -> None:
    root = tk.Tk()
    build_window(root)
    root.mainloop()


if __name__ == "__main__":
    main()
