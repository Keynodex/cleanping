"""Domain entities. Plain data only — no I/O, no framework types."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class Credential:
    """A saved provider endpoint plus the name its secret is stored under."""

    id: int | None
    name: str
    api_url: str
    model: str


@dataclass(frozen=True)
class CredentialInput:
    """What the user typed in the key dialog; the key never reaches the DB."""

    name: str
    api_url: str
    model: str
    api_key: str


@dataclass(frozen=True)
class Run:
    """One rewrite attempt, input and output kept together for the history."""

    input_text: str
    output_text: str | None
    status: str
    error_message: str | None
    duration_ms: int
    credential_name: str | None
    model: str
    prompt_text: str
    id: int | None = None
    created_at: str | None = None
    credential_id: int | None = None
