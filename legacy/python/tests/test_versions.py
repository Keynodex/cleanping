"""VersionStack: the editor's history of rewrites, with a toggle back to the original."""

import pytest

from cleanping.domain.versions import MAX_VERSIONS, VersionStack


def test_starts_with_one_empty_version():
    stack = VersionStack()
    assert (stack.current, stack.index, stack.count) == ("", 0, 1)


def test_typing_on_the_latest_edits_it_in_place():
    stack = VersionStack()
    stack.edit("rough")
    stack.edit("rough draft")
    assert (stack.current, stack.count) == ("rough draft", 1)


def test_push_adds_a_new_latest_and_keeps_the_original():
    stack = VersionStack()
    stack.edit("rough")
    stack.push("polished")
    assert (stack.current, stack.index, stack.count) == ("polished", 1, 2)
    stack.toggle_original()
    assert stack.current == "rough"


def test_pushing_identical_text_is_ignored():
    stack = VersionStack(["same"])
    assert stack.push("same") is False
    assert stack.count == 1


def test_editing_an_older_version_forks_a_new_latest():
    stack = VersionStack(["v1", "v2"], index=1)
    stack.back()
    stack.edit("v1 tweaked")
    assert stack.versions == ["v1", "v2", "v1 tweaked"]
    assert stack.index == 2


def test_back_and_forward_stop_at_the_ends():
    stack = VersionStack(["a", "b", "c"], index=2)
    assert stack.back() and stack.back()
    assert stack.back() is False and stack.current == "a"
    assert stack.forward() and stack.forward()
    assert stack.forward() is False and stack.current == "c"


def test_toggle_flips_between_original_and_latest():
    stack = VersionStack(["a", "b", "c"], index=2)
    assert stack.toggle_original() and stack.current == "a"
    assert stack.toggle_original() and stack.current == "c"


def test_toggle_with_a_single_version_does_nothing():
    assert VersionStack(["only"]).toggle_original() is False


def test_history_is_capped_and_drops_the_oldest():
    stack = VersionStack()
    for n in range(MAX_VERSIONS + 5):
        stack.push(f"v{n}")
    assert stack.count == MAX_VERSIONS
    assert stack.current == f"v{MAX_VERSIONS + 4}"
    assert stack.versions[0] != ""


def test_json_roundtrip_keeps_versions_and_position():
    stack = VersionStack(["a", "b"], index=0)
    restored = VersionStack.loads(stack.dumps())
    assert (restored.versions, restored.index) == (["a", "b"], 0)


@pytest.mark.parametrize("raw", ["", "not json", "[]", '{"versions": "x"}', '{"versions": [1]}', '{"versions": []}'])
def test_corrupt_json_falls_back_to_a_fresh_stack(raw):
    restored = VersionStack.loads(raw)
    assert (restored.versions, restored.index) == ([""], 0)


def test_out_of_range_saved_index_is_clamped():
    restored = VersionStack.loads('{"versions": ["a", "b"], "index": 9}')
    assert restored.index == 1
