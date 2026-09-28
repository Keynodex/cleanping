"""Ports (Protocols): the seams between use cases and infrastructure."""

from __future__ import annotations

from typing import Protocol

from ..domain.models import Credential, CredentialInput, Run


class Rewriter(Protocol):
    def rewrite(
        self,
        text: str,
        instructions: str,
        *,
        api_url: str,
        api_key: str,
        model: str,
    ) -> str: ...


class CredentialRepository(Protocol):
    def list(self) -> list[Credential]: ...

    def get(self, credential_id: int) -> Credential: ...

    def upsert(self, draft: CredentialInput) -> Credential: ...

    def delete(self, credential_id: int) -> None: ...


class SecretStore(Protocol):
    def get(self, name: str) -> str | None: ...

    def set(self, name: str, secret: str) -> None: ...

    def delete(self, name: str) -> None: ...


class RunRepository(Protocol):
    def add(self, run: Run) -> Run: ...

    def recent(self, limit: int = 50) -> list[Run]: ...


class PromptRepository(Protocol):
    def current(self) -> str | None: ...

    def save(self, body: str) -> None: ...


class StateRepository(Protocol):
    def get(self, key: str) -> str | None: ...

    def set(self, key: str, value: str) -> None: ...
