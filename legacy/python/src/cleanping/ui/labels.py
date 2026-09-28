"""One place that turns credentials into list labels and back to ids."""

from __future__ import annotations

from typing import Callable, Iterable

from ..domain.models import Credential


TOGGLE_LABEL = "Original"  # header button that flips original <-> latest


def credential_label(credential: Credential) -> str:
    return f"{credential.name} · {credential.model}"


class LabelIndex:
    """Labels shown in a combobox or listbox, mapped to credential ids."""

    def __init__(self) -> None:
        self._ids: dict[str, int] = {}

    def load(
        self,
        credentials: Iterable[Credential],
        label_of: Callable[[Credential], str] = credential_label,
    ) -> list[str]:
        self._ids = {label_of(c): c.id for c in credentials if c.id is not None}
        return list(self._ids)

    def id_for(self, label: str) -> int | None:
        return self._ids.get(label)

    def label_for(self, credential_id: int | None) -> str | None:
        return next((l for l, i in self._ids.items() if i == credential_id), None)
