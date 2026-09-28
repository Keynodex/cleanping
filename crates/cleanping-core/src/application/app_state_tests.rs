use super::*;
use crate::application::test_support::FakeState;

#[test]
fn selected_credential_roundtrip() {
    let state = AppState::new(FakeState::default());
    assert_eq!(state.selected_credential_id().unwrap(), None);
    state.select_credential(Some(4)).unwrap();
    assert_eq!(state.selected_credential_id().unwrap(), Some(4));
    state.select_credential(None).unwrap();
    assert_eq!(state.selected_credential_id().unwrap(), None);
}

#[test]
fn corrupt_selected_id_reads_as_none() {
    let backing = FakeState::default();
    backing
        .set("selected_credential_id", "not-a-number")
        .unwrap();
    assert_eq!(
        AppState::new(backing).selected_credential_id().unwrap(),
        None
    );
}

#[test]
fn fresh_state_starts_with_an_empty_stack() {
    let stack = AppState::new(FakeState::default()).load_versions().unwrap();
    assert_eq!((stack.versions(), stack.index()), (&[String::new()][..], 0));
}

#[test]
fn versions_roundtrip() {
    let state = AppState::new(FakeState::default());
    let mut stack = state.load_versions().unwrap();
    stack.edit("rough");
    stack.push("polished");
    state.save_versions(&stack).unwrap();
    let restored = state.load_versions().unwrap();
    assert_eq!(
        (restored.versions(), restored.index()),
        (&["rough".to_string(), "polished".to_string()][..], 1)
    );
}

#[test]
fn migrates_the_old_draft_and_result_once_then_wipes_them() {
    let backing = FakeState::default();
    backing.set("draft", "old draft").unwrap();
    backing.set("result", "old result").unwrap();
    let state = AppState::new(backing);
    let stack = state.load_versions().unwrap();
    assert_eq!(
        (stack.versions(), stack.index()),
        (&["old draft".to_string(), "old result".to_string()][..], 1)
    );
    state.save_versions(&stack).unwrap();
    assert_eq!(state.state.get("draft").unwrap().as_deref(), Some(""));
    assert_eq!(state.state.get("result").unwrap().as_deref(), Some(""));
}

#[test]
fn saving_does_not_invent_legacy_rows() {
    let state = AppState::new(FakeState::default());
    state.save_versions(&VersionStack::new()).unwrap();
    assert_eq!(state.state.get("draft").unwrap(), None);
}
