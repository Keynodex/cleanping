"""Use case service: the system prompt sent with every request."""

from __future__ import annotations

from ..domain.errors import ValidationError
from .ports import PromptRepository

DEFAULT_INSTRUCTIONS = (
    "You are a precise copy editor for a software developer's terminal prompts. "
    "Fix spelling, grammar, and clarity while retaining the author's intent, tone, "
    "technical details, and all constraints. Preserve commands, code, flags, file "
    "paths, identifiers, names, URLs, and error messages exactly. Do not execute "
    "or answer the request. Do not add facts, requirements, or explanations. "
    "Return only the edited text, with no quotes or Markdown fences. "
    "If editing would change technical meaning, leave that portion unchanged."
)


class PromptService:
    def __init__(self, prompts: PromptRepository):
        self._prompts = prompts

    def current(self) -> str:
        return self._prompts.current() or DEFAULT_INSTRUCTIONS

    def save(self, body: str) -> None:
        text = body.strip()
        if not text:
            raise ValidationError("System prompt must not be empty.")
        self._prompts.save(text)
