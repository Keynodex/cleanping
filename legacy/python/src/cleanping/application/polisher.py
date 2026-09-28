"""Use case: polish one draft and record the attempt in the history."""

from __future__ import annotations

import time
from dataclasses import dataclass

from ..domain.errors import MissingCredentialError, RewriteError
from ..domain.models import Credential, Run
from ..domain.validation import is_local_url
from .ports import Rewriter, RunRepository, SecretStore


@dataclass(frozen=True)
class PolishResult:
    output_text: str | None
    error_message: str | None
    run: Run

    @property
    def ok(self) -> bool:
        return self.error_message is None


class PolishText:
    """Runs the provider call, then persists an ok/error row either way."""

    def __init__(self, rewriter: Rewriter, runs: RunRepository, secrets: SecretStore):
        self._rewriter = rewriter
        self._runs = runs
        self._secrets = secrets

    def __call__(self, text: str, instructions: str, credential: Credential) -> PolishResult:
        api_key = self._secrets.get(credential.name)
        if not api_key and not is_local_url(credential.api_url):
            raise MissingCredentialError(
                f"No API key saved for “{credential.name}”. Open API keys (Alt+K)."
            )
        started = time.monotonic()
        try:
            output: str | None = self._rewriter.rewrite(
                text,
                instructions,
                api_url=credential.api_url,
                api_key=api_key or "",
                model=credential.model,
            )
            error: str | None = None
        except RewriteError as exc:
            output, error = None, str(exc)
        duration_ms = int((time.monotonic() - started) * 1000)
        run = self._runs.add(
            Run(
                input_text=text,
                output_text=output,
                status="error" if error else "ok",
                error_message=error,
                duration_ms=duration_ms,
                credential_name=credential.name,
                model=credential.model,
                prompt_text=instructions,
                credential_id=credential.id,
            )
        )
        return PolishResult(output_text=output, error_message=error, run=run)
