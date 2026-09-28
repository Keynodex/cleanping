"""HTTP adapter: URL gate, response parsing, safe error mapping."""

import io
import json
from email.message import Message
from urllib.error import HTTPError

import pytest

from cleanping.domain.errors import RewriteError, ValidationError
from cleanping.infrastructure import http_rewriter
from cleanping.infrastructure.http_rewriter import OpenAIRewriter


class _FakeResponse:
    def __init__(self, payload: bytes):
        self._stream = io.BytesIO(payload)

    def __enter__(self):
        return self

    def __exit__(self, *exc_info):
        return False

    def read(self, *args):
        return self._stream.read(*args)


class _FakeOpener:
    def __init__(self, outcome):
        self._outcome = outcome
        self.requests = []

    def open(self, request, timeout=None):
        self.requests.append(request)
        if isinstance(self._outcome, Exception):
            raise self._outcome
        return _FakeResponse(self._outcome)


def _install(monkeypatch, outcome):
    opener = _FakeOpener(outcome)
    monkeypatch.setattr(http_rewriter, "build_opener", lambda *handlers: opener)
    return opener


_OK_PAYLOAD = json.dumps(
    {"choices": [{"message": {"role": "assistant", "content": "  Fixed text.  "}}]}
).encode()


class TestOpenAIRewriter:
    def test_rejects_unsafe_url_before_any_request(self, monkeypatch):
        opener = _install(monkeypatch, _OK_PAYLOAD)
        rewriter = OpenAIRewriter()
        with pytest.raises(ValidationError):
            rewriter.rewrite(
                "text",
                "instructions",
                api_url="http://remote.example.com/v1/chat/completions",
                api_key="sk-x",
                model="m",
            )
        assert opener.requests == []

    def test_returns_trimmed_content(self, monkeypatch):
        _install(monkeypatch, _OK_PAYLOAD)
        result = OpenAIRewriter().rewrite(
            "  pleae fix  ",
            "Fix it.",
            api_url="https://api.example.com/v1/chat/completions",
            api_key="sk-x",
            model="m",
        )
        assert result == "Fixed text."

    def test_sends_system_prompt_and_key_header(self, monkeypatch):
        opener = _install(monkeypatch, _OK_PAYLOAD)
        OpenAIRewriter().rewrite(
            "draft",
            "Fix it.",
            api_url="https://api.example.com/v1/chat/completions",
            api_key="sk-x",
            model="m",
        )
        request = opener.requests[0]
        assert request.get_header("Authorization") == "Bearer sk-x"
        body = json.loads(request.data)
        assert body["messages"][0] == {"role": "system", "content": "Fix it."}
        assert body["messages"][1] == {"role": "user", "content": "draft"}

    def test_http_error_maps_to_safe_message_without_body(self, monkeypatch):
        error = HTTPError(
            "https://api.example.com",
            401,
            "Unauthorized",
            Message(),
            io.BytesIO(b"secret server detail"),
        )
        _install(monkeypatch, error)
        with pytest.raises(RewriteError, match="HTTP 401") as excinfo:
            OpenAIRewriter().rewrite(
                "draft",
                "Fix it.",
                api_url="https://api.example.com/v1/chat/completions",
                api_key="sk-x",
                model="m",
            )
        assert "secret server detail" not in str(excinfo.value)

    def test_malformed_payload_maps_to_rewrite_error(self, monkeypatch):
        _install(monkeypatch, b'{"unexpected": true}')
        with pytest.raises(RewriteError, match="edited text"):
            OpenAIRewriter().rewrite(
                "draft",
                "Fix it.",
                api_url="https://api.example.com/v1/chat/completions",
                api_key="sk-x",
                model="m",
            )

    def test_empty_edit_maps_to_rewrite_error(self, monkeypatch):
        payload = json.dumps(
            {"choices": [{"message": {"content": "   "}}]}
        ).encode()
        _install(monkeypatch, payload)
        with pytest.raises(RewriteError, match="empty"):
            OpenAIRewriter().rewrite(
                "draft",
                "Fix it.",
                api_url="https://api.example.com/v1/chat/completions",
                api_key="sk-x",
                model="m",
            )
