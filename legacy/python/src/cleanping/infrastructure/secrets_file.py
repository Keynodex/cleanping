"""JSON secret store: values live in one 0600 file, never logged or echoed."""

from __future__ import annotations

import json
import os
from pathlib import Path

from ..domain.errors import CleanpingError


class JsonSecretStore:
    """File-backed SecretStore port; keys are credential names."""

    def __init__(self, path: Path):
        self._path = Path(path)

    def get(self, name: str) -> str | None:
        return self._read().get(name)

    def set(self, name: str, secret: str) -> None:
        data = self._read()
        data[name] = secret
        self._write(data)

    def delete(self, name: str) -> None:
        data = self._read()
        if name in data:
            data.pop(name)
            self._write(data)

    def _read(self) -> dict[str, str]:
        try:
            raw = self._path.read_text(encoding="utf-8")
        except FileNotFoundError:
            return {}
        except OSError as exc:
            raise CleanpingError(
                f"Cannot read the secret store: {exc.strerror or exc}."
            ) from exc
        try:
            data = json.loads(raw)
        except json.JSONDecodeError as exc:
            raise CleanpingError(
                "Secret store is not valid JSON; fix or remove it before saving keys."
            ) from exc
        if not isinstance(data, dict):
            raise CleanpingError("Secret store must contain a JSON object.")
        return data

    def _write(self, data: dict[str, str]) -> None:
        directory = self._path.parent
        directory.mkdir(parents=True, exist_ok=True, mode=0o700)
        try:
            os.chmod(directory, 0o700)
        except OSError:
            pass
        tmp_path = self._path.with_name(self._path.name + ".tmp")
        fd = os.open(tmp_path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
        with os.fdopen(fd, "w", encoding="utf-8") as handle:
            json.dump(data, handle, indent=2)
            handle.write("\n")
        os.replace(tmp_path, self._path)
