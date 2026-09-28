"""Domain errors — user-facing failure categories raised by the core."""


class CleanpingError(Exception):
    """Base class for every expected cleanping failure."""


class ValidationError(CleanpingError):
    """A user-supplied value violates a domain invariant."""


class NotFoundError(CleanpingError):
    """A stored record referenced by id no longer exists."""


class MissingCredentialError(CleanpingError):
    """The request cannot run because no API key is available for it."""


class RewriteError(CleanpingError):
    """The provider could not produce an edited draft (safe message only)."""
