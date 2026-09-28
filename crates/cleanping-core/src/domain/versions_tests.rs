use super::*;

fn stack(versions: &[&str], index: usize) -> VersionStack {
    VersionStack::from_parts(
        versions.iter().map(|v| (*v).to_string()).collect(),
        Some(index),
    )
}

#[test]
fn starts_with_one_empty_version() {
    let s = VersionStack::new();
    assert_eq!((s.current(), s.index(), s.count()), ("", 0, 1));
}

#[test]
fn typing_on_the_latest_edits_it_in_place() {
    let mut s = VersionStack::new();
    s.edit("rough");
    s.edit("rough draft");
    assert_eq!((s.current(), s.count()), ("rough draft", 1));
}

#[test]
fn push_adds_a_new_latest_and_keeps_the_original() {
    let mut s = VersionStack::new();
    s.edit("rough");
    assert!(s.push("polished"));
    assert_eq!((s.current(), s.index(), s.count()), ("polished", 1, 2));
    s.toggle_original();
    assert_eq!(s.current(), "rough");
}

#[test]
fn pushing_identical_text_is_ignored() {
    let mut s = VersionStack::from_parts(vec!["same".into()], None);
    assert!(!s.push("same"));
    assert_eq!(s.count(), 1);
}

#[test]
fn editing_an_older_version_forks_a_new_latest() {
    let mut s = stack(&["v1", "v2"], 1);
    s.back();
    s.edit("v1 tweaked");
    assert_eq!(s.versions(), ["v1", "v2", "v1 tweaked"]);
    assert_eq!(s.index(), 2);
}

#[test]
fn back_and_forward_stop_at_the_ends() {
    let mut s = stack(&["a", "b", "c"], 2);
    assert!(s.back() && s.back());
    assert!(!s.back());
    assert_eq!(s.current(), "a");
    assert!(s.forward() && s.forward());
    assert!(!s.forward());
    assert_eq!(s.current(), "c");
}

#[test]
fn toggle_flips_between_original_and_latest() {
    let mut s = stack(&["a", "b", "c"], 2);
    assert!(s.toggle_original());
    assert_eq!(s.current(), "a");
    assert!(s.toggle_original());
    assert_eq!(s.current(), "c");
}

#[test]
fn toggle_with_a_single_version_does_nothing() {
    assert!(!VersionStack::from_parts(vec!["only".into()], None).toggle_original());
}

#[test]
fn history_is_capped_and_drops_the_oldest() {
    let mut s = VersionStack::new();
    for n in 0..MAX_VERSIONS + 5 {
        s.push(&format!("v{n}"));
    }
    assert_eq!(s.count(), MAX_VERSIONS);
    assert_eq!(s.current(), format!("v{}", MAX_VERSIONS + 4));
    assert_ne!(s.versions()[0], "");
}

#[test]
fn json_roundtrip_keeps_versions_and_position() {
    let restored = VersionStack::loads(&stack(&["a", "b"], 0).dumps());
    assert_eq!(
        (restored.versions(), restored.index()),
        (&["a".to_string(), "b".to_string()][..], 0)
    );
}

#[test]
fn corrupt_json_falls_back_to_a_fresh_stack() {
    for raw in [
        "",
        "not json",
        "[]",
        r#"{"versions": "x"}"#,
        r#"{"versions": [1]}"#,
        r#"{"versions": []}"#,
    ] {
        let s = VersionStack::loads(raw);
        assert_eq!(
            (s.versions(), s.index()),
            (&[String::new()][..], 0),
            "{raw}"
        );
    }
}

#[test]
fn out_of_range_saved_index_is_clamped() {
    assert_eq!(
        VersionStack::loads(r#"{"versions": ["a", "b"], "index": 9}"#).index(),
        1
    );
}

#[test]
fn reads_the_json_the_python_release_wrote() {
    let s = VersionStack::loads(r#"{"versions": ["rough", "Polished"], "index": 1}"#);
    assert_eq!((s.current(), s.count()), ("Polished", 2));
}
