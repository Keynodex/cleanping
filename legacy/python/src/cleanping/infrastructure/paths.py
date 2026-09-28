"""XDG locations for cleanping runtime data (never inside the repo)."""

import os
from pathlib import Path


def _base(env_var: str, fallback: Path) -> Path:
    return Path(os.environ.get(env_var) or fallback)


def config_dir() -> Path:
    return _base("XDG_CONFIG_HOME", Path.home() / ".config") / "cleanping"


def data_dir() -> Path:
    return _base("XDG_DATA_HOME", Path.home() / ".local" / "share") / "cleanping"


def secrets_path() -> Path:
    return config_dir() / "secrets.json"


def database_path() -> Path:
    return data_dir() / "cleanping.db"
