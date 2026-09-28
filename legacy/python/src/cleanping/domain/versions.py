"""Editor history: the user's text and every rewrite of it. Plain data, no I/O."""

from __future__ import annotations

import json

MAX_VERSIONS = 20


class VersionStack:
    """Ordered versions of one piece of text plus the one currently shown."""

    def __init__(self, versions: list[str] | None = None, index: int | None = None):
        self._versions = list(versions) if versions else [""]
        last = len(self._versions) - 1
        self._index = last if index is None else max(0, min(index, last))

    @property
    def versions(self) -> list[str]:
        return list(self._versions)

    @property
    def current(self) -> str:
        return self._versions[self._index]

    @property
    def index(self) -> int:
        return self._index

    @property
    def count(self) -> int:
        return len(self._versions)

    def edit(self, text: str) -> None:
        """The user changed the shown text: update the latest, or fork from an older one."""
        if text == self.current:
            return
        if self._index == len(self._versions) - 1:
            self._versions[-1] = text
        else:
            self._append(text)

    def push(self, text: str) -> bool:
        """Add a rewrite as the new latest. False when it equals what is already shown."""
        if text == self.current and self._index == len(self._versions) - 1:
            return False
        self._append(text)
        return True

    def back(self) -> bool:
        return self._move(self._index - 1)

    def forward(self) -> bool:
        return self._move(self._index + 1)

    def toggle_original(self) -> bool:
        """Flip between the oldest kept version and the latest."""
        if len(self._versions) < 2:
            return False
        return self._move(0 if self._index else len(self._versions) - 1)

    def dumps(self) -> str:
        return json.dumps({"versions": self._versions, "index": self._index})

    @classmethod
    def loads(cls, raw: str) -> VersionStack:
        try:
            data = json.loads(raw)
            versions, index = data["versions"], data.get("index")
            if not isinstance(versions, list) or not versions or not all(
                isinstance(v, str) for v in versions
            ):
                return cls()
            return cls(versions, index if isinstance(index, int) else None)
        except (ValueError, TypeError, KeyError, AttributeError):
            return cls()

    def _append(self, text: str) -> None:
        self._versions.append(text)
        if len(self._versions) > MAX_VERSIONS:
            self._versions.pop(0)
        self._index = len(self._versions) - 1

    def _move(self, target: int) -> bool:
        if not 0 <= target < len(self._versions) or target == self._index:
            return False
        self._index = target
        return True
