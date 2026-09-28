"""HTTP adapter for OpenAI-compatible chat completions (stdlib urllib)."""

from __future__ import annotations

import json
from urllib.error import HTTPError, URLError
from urllib.request import HTTPRedirectHandler, Request, build_opener

from ..domain.errors import RewriteError
from ..domain.validation import validate_api_url

DEFAULT_TIMEOUT_SECONDS = 180.0  # cold local models can take a while to load


class _NoRedirects(HTTPRedirectHandler):
    """A redirect must never forward the API key or the draft to another host."""

    def redirect_request(self, request, fp, code, msg, headers, newurl):
        return None


class OpenAIRewriter:
    """Rewriter port implementation for OpenAI-compatible chat completions."""

    def __init__(self, timeout: float = DEFAULT_TIMEOUT_SECONDS):
        self._timeout = timeout

    def rewrite(
        self,
        text: str,
        instructions: str,
        *,
        api_url: str,
        api_key: str,
        model: str,
    ) -> str:
        validate_api_url(api_url)  # never send the key to a URL the domain rejects
        payload = json.dumps(
            {
                "model": model,
                "messages": [
                    {"role": "system", "content": instructions},
                    {"role": "user", "content": text},
                ],
            }
        ).encode("utf-8")
        request = Request(
            api_url,
            data=payload,
            headers={
                "Authorization": f"Bearer {api_key}",
                "Content-Type": "application/json",
            },
            method="POST",
        )
        try:
            with build_opener(_NoRedirects).open(request, timeout=self._timeout) as response:
                data = json.load(response)
        except HTTPError as exc:
            # Never echo server bodies; they can contain private prompts.
            raise RewriteError(f"API returned HTTP {exc.code}.") from exc
        except URLError as exc:
            raise RewriteError(f"Could not reach the API: {exc.reason}.") from exc
        except (TimeoutError, OSError) as exc:
            raise RewriteError(f"Could not reach the API: {exc}.") from exc
        try:
            result = data["choices"][0]["message"]["content"].strip()
        except (KeyError, IndexError, AttributeError, TypeError) as exc:
            raise RewriteError("API response did not contain edited text.") from exc
        if not result:
            raise RewriteError("API returned an empty edit; nothing was copied.")
        return result
