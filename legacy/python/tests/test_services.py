"""Service use cases: credential lifecycle, prompt handling, app state."""

import pytest

from cleanping.application.app_state import AppState
from cleanping.application.credentials import CredentialService
from cleanping.application.prompts import DEFAULT_INSTRUCTIONS, PromptService
from cleanping.domain.errors import NotFoundError, ValidationError
from cleanping.domain.models import Credential, CredentialInput


class FakeCredentials:
    def __init__(self):
        self.rows: dict[int, Credential] = {}
        self.next_id = 1

    def list(self):
        return sorted(self.rows.values(), key=lambda item: item.name.lower())

    def get(self, credential_id):
        if credential_id not in self.rows:
            raise NotFoundError(f"Credential {credential_id} no longer exists.")
        return self.rows[credential_id]

    def upsert(self, draft):
        for stored in self.rows.values():
            if stored.name == draft.name:
                updated = Credential(stored.id, draft.name, draft.api_url, draft.model)
                self.rows[stored.id] = updated
                return updated
        created = Credential(self.next_id, draft.name, draft.api_url, draft.model)
        self.rows[self.next_id] = created
        self.next_id += 1
        return created

    def delete(self, credential_id):
        self.rows.pop(credential_id, None)


class FakeSecrets:
    def __init__(self):
        self.stored = {}

    def get(self, name):
        return self.stored.get(name)

    def set(self, name, secret):
        self.stored[name] = secret

    def delete(self, name):
        self.stored.pop(name, None)


class FakePrompts:
    def __init__(self):
        self.body = None

    def current(self):
        return self.body

    def save(self, body):
        self.body = body


class FakeState:
    def __init__(self):
        self.values = {}

    def get(self, key):
        return self.values.get(key)

    def set(self, key, value):
        self.values[key] = value


def _input(key="sk-x", name="OpenAI"):
    return CredentialInput(
        name=name,
        api_url="https://api.openai.com/v1/chat/completions",
        model="gpt-4o-mini",
        api_key=key,
    )


class TestCredentialService:
    def test_save_new_stores_secret(self):
        secrets = FakeSecrets()
        service = CredentialService(FakeCredentials(), secrets)
        credential = service.save(_input())
        assert credential.name == "OpenAI"
        assert secrets.get("OpenAI") == "sk-x"

    def test_save_existing_with_blank_key_keeps_secret(self):
        secrets = FakeSecrets()
        service = CredentialService(FakeCredentials(), secrets)
        service.save(_input())
        service.save(_input(key=""))
        assert secrets.get("OpenAI") == "sk-x"

    def test_save_new_with_blank_key_is_rejected(self):
        service = CredentialService(FakeCredentials(), FakeSecrets())
        with pytest.raises(ValidationError, match="API key"):
            service.save(_input(key="   "))

    def test_save_new_local_key_needs_no_secret(self):
        secrets = FakeSecrets()
        service = CredentialService(FakeCredentials(), secrets)
        draft = CredentialInput(
            name="Ollama",
            api_url="http://127.0.0.1:11434/v1/chat/completions",
            model="qwen2.5:7b",
            api_key="",
        )
        credential = service.save(draft)
        assert credential.name == "Ollama"
        assert secrets.get("Ollama") is None

    def test_delete_removes_row_and_secret(self):
        secrets = FakeSecrets()
        service = CredentialService(FakeCredentials(), secrets)
        credential = service.save(_input())
        service.delete(credential.id)
        assert secrets.get("OpenAI") is None
        assert service.list() == []


class TestPromptService:
    def test_current_falls_back_to_default(self):
        assert PromptService(FakePrompts()).current() == DEFAULT_INSTRUCTIONS

    def test_save_then_current_returns_saved(self):
        service = PromptService(FakePrompts())
        service.save("  Fix typos only.  ")
        assert service.current() == "Fix typos only."

    def test_empty_save_is_rejected(self):
        with pytest.raises(ValidationError):
            PromptService(FakePrompts()).save("   ")


class TestAppState:
    def test_draft_roundtrip(self):
        state = AppState(FakeState())
        assert state.load_draft() == ""
        state.save_draft("half-written prompt")
        assert state.load_draft() == "half-written prompt"

    def test_selected_credential_roundtrip(self):
        state = AppState(FakeState())
        assert state.selected_credential_id() is None
        state.select_credential(4)
        assert state.selected_credential_id() == 4
        state.select_credential(None)
        assert state.selected_credential_id() is None

    def test_corrupt_selected_id_reads_as_none(self):
        backing = FakeState()
        backing.set("selected_credential_id", "not-a-number")
        assert AppState(backing).selected_credential_id() is None

    def test_geometry_roundtrip(self):
        state = AppState(FakeState())
        state.save_geometry("720x380+40+40")
        assert state.load_geometry() == "720x380+40+40"


class TestThemePreference:
    def test_unset_theme_reads_as_none(self):
        assert AppState(FakeState()).load_theme() is None

    def test_theme_roundtrip(self):
        state = AppState(FakeState())
        state.save_theme("dark")
        assert state.load_theme() == "dark"

    def test_unknown_theme_value_reads_as_none(self):
        backing = FakeState()
        backing.set("theme", "neon")
        assert AppState(backing).load_theme() is None


class TestVersionsState:
    def test_fresh_state_starts_with_an_empty_stack(self):
        stack = AppState(FakeState()).load_versions()
        assert (stack.versions, stack.index) == ([""], 0)

    def test_roundtrip(self):
        state = AppState(FakeState())
        stack = state.load_versions()
        stack.edit("rough")
        stack.push("polished")
        state.save_versions(stack)
        restored = state.load_versions()
        assert (restored.versions, restored.index) == (["rough", "polished"], 1)

    def test_migrates_the_old_draft_and_result_once(self):
        backing = FakeState()
        backing.set("draft", "old draft")
        backing.set("result", "old result")
        stack = AppState(backing).load_versions()
        assert (stack.versions, stack.index) == (["old draft", "old result"], 1)


class TestVersionsMigrationCleanup:
    def test_saving_versions_wipes_the_legacy_draft_and_result(self):
        backing = FakeState()
        backing.set("draft", "old draft")
        backing.set("result", "old result")
        state = AppState(backing)
        state.save_versions(state.load_versions())
        assert backing.get("draft") == ""
        assert backing.get("result") == ""

    def test_saving_versions_does_not_invent_legacy_rows(self):
        backing = FakeState()
        state = AppState(backing)
        state.save_versions(state.load_versions())
        assert backing.get("draft") is None and backing.get("result") is None


class TestThemeValidation:
    def test_unknown_theme_is_rejected(self):
        state = AppState(FakeState())
        with pytest.raises(ValidationError):
            state.save_theme("neon")
        assert state.load_theme() is None
