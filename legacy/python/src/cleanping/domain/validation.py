"""Pure invariants for provider URLs and credential fields."""

from __future__ import annotations

from urllib.parse import urlparse

from .errors import ValidationError
from .models import CredentialInput

_LOCAL_HOSTS = {"localhost", "127.0.0.1", "::1"}


def is_local_url(url: str) -> bool:
    """True when the URL points at this machine (no key needs to travel)."""
    return urlparse(url.strip()).hostname in _LOCAL_HOSTS


def validate_api_url(url: str) -> str:
    """Return the URL unchanged when it is safe to send an API key to."""
    normalized = url.strip()
    parsed = urlparse(normalized)
    secure = parsed.scheme == "https"
    local_http = parsed.scheme == "http" and parsed.hostname in _LOCAL_HOSTS
    if not secure and not local_http:
        raise ValidationError(
            "API URL must use HTTPS (HTTP is allowed only for localhost)."
        )
    if not parsed.hostname or parsed.username or parsed.password:
        raise ValidationError(
            "API URL must be a valid URL without embedded credentials."
        )
    return normalized


def validate_credential_fields(draft: CredentialInput) -> CredentialInput:
    """Validate name, URL and model; return a whitespace-normalized copy."""
    name = draft.name.strip()
    if not name:
        raise ValidationError("Give this key a name.")
    model = draft.model.strip()
    if not model:
        raise ValidationError("model must not be empty.")
    return CredentialInput(
        name=name,
        api_url=validate_api_url(draft.api_url),
        model=model,
        api_key=draft.api_key.strip(),
    )
