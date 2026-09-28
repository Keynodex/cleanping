"""Invariants for API URLs and credential fields (domain)."""

import pytest

from cleanping.domain.errors import ValidationError
from cleanping.domain.models import CredentialInput
from cleanping.domain.validation import (
    is_local_url,
    validate_api_url,
    validate_credential_fields,
)


class TestIsLocalUrl:
    def test_loopback_forms_are_local(self):
        for url in (
            "http://localhost:11434/v1/chat/completions",
            "http://127.0.0.1:11434/v1/chat/completions",
            "http://[::1]:11434/v1/chat/completions",
        ):
            assert is_local_url(url) is True

    def test_remote_hosts_are_not_local(self):
        assert is_local_url("https://api.deepseek.com/v1/chat/completions") is False

    def test_garbage_is_not_local(self):
        assert is_local_url("not a url") is False


class TestValidateApiUrl:
    def test_accepts_https(self):
        assert validate_api_url("https://api.openai.com/v1/chat/completions") == (
            "https://api.openai.com/v1/chat/completions"
        )

    def test_accepts_http_localhost_and_loopback(self):
        for host in ("localhost", "127.0.0.1", "[::1]"):
            url = f"http://{host}:11434/v1/chat/completions"
            assert validate_api_url(url) == url

    def test_strips_surrounding_whitespace(self):
        assert validate_api_url("  https://example.com/v1  ") == "https://example.com/v1"

    def test_rejects_plain_http_for_remote_host(self):
        with pytest.raises(ValidationError, match="HTTPS"):
            validate_api_url("http://api.example.com/v1/chat/completions")

    def test_rejects_embedded_credentials(self):
        with pytest.raises(ValidationError, match="credentials"):
            validate_api_url("https://user:pass@example.com/v1")

    def test_rejects_missing_host(self):
        with pytest.raises(ValidationError):
            validate_api_url("https:///v1/chat/completions")

    def test_rejects_non_http_scheme(self):
        with pytest.raises(ValidationError):
            validate_api_url("ftp://example.com/v1")


class TestValidateCredentialFields:
    def _draft(self, **overrides):
        values = {
            "name": "OpenAI",
            "api_url": "https://api.openai.com/v1/chat/completions",
            "model": "gpt-4o-mini",
            "api_key": "sk-test",
        }
        values.update(overrides)
        return CredentialInput(**values)

    def test_normalizes_whitespace(self):
        result = validate_credential_fields(self._draft(name="  OpenAI  ", model=" x "))
        assert result.name == "OpenAI"
        assert result.model == "x"

    def test_requires_name(self):
        with pytest.raises(ValidationError, match="name"):
            validate_credential_fields(self._draft(name="   "))

    def test_requires_model(self):
        with pytest.raises(ValidationError, match="model"):
            validate_credential_fields(self._draft(model=""))

    def test_propagates_url_rules(self):
        with pytest.raises(ValidationError):
            validate_credential_fields(self._draft(api_url="http://remote.example.com/v1"))
