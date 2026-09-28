"""Secret store: roundtrip, file modes, safe failure on corrupt data."""

import json
import stat

import pytest

from cleanping.domain.errors import CleanpingError
from cleanping.infrastructure.secrets_file import JsonSecretStore


def _mode(path):
    return stat.S_IMODE(path.stat().st_mode)


class TestJsonSecretStore:
    def test_missing_store_reads_as_empty(self, tmp_path):
        store = JsonSecretStore(tmp_path / "nested" / "secrets.json")
        assert store.get("anything") is None

    def test_set_get_delete_roundtrip(self, tmp_path):
        store = JsonSecretStore(tmp_path / "secrets.json")
        store.set("OpenAI", "sk-secret")
        assert store.get("OpenAI") == "sk-secret"
        store.delete("OpenAI")
        assert store.get("OpenAI") is None

    def test_delete_missing_name_is_noop(self, tmp_path):
        store = JsonSecretStore(tmp_path / "secrets.json")
        store.delete("never-saved")

    def test_file_and_directory_are_private(self, tmp_path):
        path = tmp_path / "config" / "secrets.json"
        JsonSecretStore(path).set("name", "value")
        assert _mode(path) == 0o600
        assert _mode(path.parent) == 0o700

    def test_rewrite_keeps_other_secrets(self, tmp_path):
        store = JsonSecretStore(tmp_path / "secrets.json")
        store.set("a", "1")
        store.set("b", "2")
        store.set("a", "updated")
        assert json.loads((tmp_path / "secrets.json").read_text()) == {
            "a": "updated",
            "b": "2",
        }

    def test_corrupt_json_fails_loudly(self, tmp_path):
        path = tmp_path / "secrets.json"
        path.write_text("{not json")
        store = JsonSecretStore(path)
        with pytest.raises(CleanpingError, match="not valid JSON"):
            store.set("name", "value")
