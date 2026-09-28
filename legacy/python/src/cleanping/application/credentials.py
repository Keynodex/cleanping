"""Use case service: save, list and delete provider credentials."""

from __future__ import annotations

from ..domain.errors import ValidationError
from ..domain.models import Credential, CredentialInput
from ..domain.validation import is_local_url, validate_credential_fields
from .ports import CredentialRepository, SecretStore


class CredentialService:
    """Keeps the DB row (name/url/model) and the secret file in sync."""

    def __init__(self, credentials: CredentialRepository, secrets: SecretStore):
        self._credentials = credentials
        self._secrets = secrets

    def list(self) -> list[Credential]:
        return self._credentials.list()

    def get(self, credential_id: int) -> Credential:
        return self._credentials.get(credential_id)

    def save(self, draft: CredentialInput) -> Credential:
        """Blank key means "keep the stored one"; only local URLs may stay keyless."""
        normalized = validate_credential_fields(draft)
        if normalized.api_key:
            self._secrets.set(normalized.name, normalized.api_key)
        elif not self._secrets.get(normalized.name) and not is_local_url(
            normalized.api_url
        ):
            raise ValidationError("Enter an API key.")
        return self._credentials.upsert(normalized)

    def delete(self, credential_id: int) -> None:
        credential = self._credentials.get(credential_id)
        self._credentials.delete(credential_id)
        self._secrets.delete(credential.name)
