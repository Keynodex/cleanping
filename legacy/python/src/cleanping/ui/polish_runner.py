"""Run one rewrite at a time off the Tk thread; results return on the UI thread."""

from __future__ import annotations

import queue
import threading
import tkinter as tk
import traceback
from dataclasses import dataclass
from typing import Callable

from ..application.polisher import PolishResult, PolishText
from ..domain.errors import CleanpingError
from ..domain.models import Credential

_POLL_MS = 80


@dataclass(frozen=True)
class PolishOutcome:
    """Exactly one of `result` (the run happened) or `error` (it never started)."""

    result: PolishResult | None = None
    error: str | None = None


class PolishRunner:
    """Owns the worker thread, the outcome queue and the Tk poll loop."""

    def __init__(self, root: tk.Misc, polish: PolishText, on_outcome: Callable[[PolishOutcome], None]):
        self._root = root
        self._polish = polish
        self._on_outcome = on_outcome
        self._pending: queue.Queue[PolishOutcome] = queue.Queue()
        self.busy = False
        self._root.after(_POLL_MS, self._poll)

    def start(self, source: str, instructions: str, credential: Credential) -> bool:
        """False when a rewrite is already running; true when this one starts."""
        if self.busy:
            return False
        self.busy = True
        threading.Thread(
            target=self._work, args=(source, instructions, credential), daemon=True
        ).start()
        return True

    def _work(self, source: str, instructions: str, credential: Credential) -> None:
        try:
            outcome = PolishOutcome(result=self._polish(source, instructions, credential))
        except CleanpingError as exc:
            outcome = PolishOutcome(error=str(exc))
        except Exception:  # unexpected bugs must surface, not hang the window
            traceback.print_exc()
            outcome = PolishOutcome(error="Unexpected error; see terminal log.")
        self._pending.put(outcome)

    def _poll(self) -> None:
        try:
            while True:
                outcome = self._pending.get_nowait()
                self.busy = False
                self._on_outcome(outcome)
        except queue.Empty:
            pass
        self._root.after(_POLL_MS, self._poll)
