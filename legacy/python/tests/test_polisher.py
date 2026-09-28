"""Polish use case: outcome shape, run recording, missing-key handling."""

import pytest

from cleanping.application.polisher import PolishText
from cleanping.domain.errors import MissingCredentialError, RewriteError
from cleanping.domain.models import Credential, Run


class FakeRewriter:
    def __init__(self, result=None, error=None):
        self.result = result
        self.error = error
        self.calls = []

    def rewrite(self, text, instructions, *, api_url, api_key, model):
        self.calls.append((text, instructions, api_url, api_key, model))
        if self.error:
            raise self.error
        return self.result


class FakeSecrets:
    def __init__(self, stored=None):
        self.stored = stored or {}

    def get(self, name):
        return self.stored.get(name)

    def set(self, name, secret):
        self.stored[name] = secret

    def delete(self, name):
        self.stored.pop(name, None)


class FakeRuns:
    def __init__(self):
        self.rows = []

    def add(self, run):
        stored = Run(
            input_text=run.input_text,
            output_text=run.output_text,
            status=run.status,
            error_message=run.error_message,
            duration_ms=run.duration_ms,
            credential_name=run.credential_name,
            model=run.model,
            prompt_text=run.prompt_text,
            id=len(self.rows) + 1,
            created_at="2026-09-28T00:00:00+00:00",
            credential_id=run.credential_id,
        )
        self.rows.append(stored)
        return stored

    def recent(self, limit=50):
        return list(reversed(self.rows))[:limit]


def _credential():
    return Credential(id=7, name="OpenAI", api_url="https://api.example.com/v1", model="m")


class TestPolishText:
    def _use_case(self, rewriter, stored=None):
        runs = FakeRuns()
        secrets = FakeSecrets(stored)
        return PolishText(rewriter, runs, secrets), runs

    def test_success_records_ok_run_and_returns_output(self):
        rewriter = FakeRewriter(result="Fixed text.")
        use_case, runs = self._use_case(rewriter, {"OpenAI": "sk-x"})
        result = use_case("pleae fix", "Fix it.", _credential())
        assert result.ok
        assert result.output_text == "Fixed text."
        assert runs.rows[0].status == "ok"
        assert runs.rows[0].credential_name == "OpenAI"
        assert runs.rows[0].credential_id == 7
        assert runs.rows[0].prompt_text == "Fix it."

    def test_provider_error_records_error_run_and_returns_message(self):
        rewriter = FakeRewriter(error=RewriteError("API returned HTTP 401."))
        use_case, runs = self._use_case(rewriter, {"OpenAI": "sk-x"})
        result = use_case("draft", "Fix it.", _credential())
        assert not result.ok
        assert result.error_message == "API returned HTTP 401."
        assert runs.rows[0].status == "error"
        assert runs.rows[0].output_text is None

    def test_missing_key_raises_and_records_nothing(self):
        use_case, runs = self._use_case(FakeRewriter(result="x"))
        with pytest.raises(MissingCredentialError):
            use_case("draft", "Fix it.", _credential())
        assert runs.rows == []

    def test_local_url_without_key_runs_with_empty_key(self):
        rewriter = FakeRewriter(result="ok")
        use_case, runs = self._use_case(rewriter)
        local = Credential(
            id=8,
            name="Ollama",
            api_url="http://127.0.0.1:11434/v1/chat/completions",
            model="qwen2.5:7b",
        )
        result = use_case("draft", "Fix it.", local)
        assert result.ok
        assert rewriter.calls[0][3] == ""
        assert runs.rows[0].status == "ok"

    def test_key_is_looked_up_by_credential_name(self):
        rewriter = FakeRewriter(result="ok")
        use_case, _ = self._use_case(rewriter, {"OpenAI": "sk-secret"})
        use_case("draft", "Fix it.", _credential())
        assert rewriter.calls[0][3] == "sk-secret"
