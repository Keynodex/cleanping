"""UTC timestamps in one format everywhere."""

from datetime import datetime, timezone


def utc_now() -> str:
    """ISO-8601 UTC, second precision, e.g. 2026-09-28T18:30:00+00:00."""
    return datetime.now(timezone.utc).isoformat(timespec="seconds")
