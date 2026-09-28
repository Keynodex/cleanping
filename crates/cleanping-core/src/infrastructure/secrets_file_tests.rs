use super::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;

fn mode(path: &std::path::Path) -> u32 {
    fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[test]
fn missing_store_reads_as_empty() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonSecretStore::new(dir.path().join("nested/secrets.json"));
    assert_eq!(store.get("anything").unwrap(), None);
}

#[test]
fn set_get_delete_roundtrip_and_delete_missing_is_noop() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonSecretStore::new(dir.path().join("secrets.json"));
    store.set("OpenAI", "sk-secret").unwrap();
    assert_eq!(store.get("OpenAI").unwrap().as_deref(), Some("sk-secret"));
    store.delete("OpenAI").unwrap();
    assert_eq!(store.get("OpenAI").unwrap(), None);
    store.delete("never-saved").unwrap();
}

#[test]
fn file_and_directory_are_private() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config/secrets.json");
    JsonSecretStore::new(&path).set("name", "value").unwrap();
    assert_eq!(mode(&path), 0o600);
    assert_eq!(mode(path.parent().unwrap()), 0o700);
}

#[test]
fn a_stale_temp_file_with_a_loose_mode_is_never_reused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("secrets.json");
    let leftover = dir.path().join("secrets.json.tmp");
    fs::write(&leftover, "stale").unwrap();
    fs::set_permissions(&leftover, fs::Permissions::from_mode(0o644)).unwrap();
    JsonSecretStore::new(&path).set("a", "1").unwrap();
    assert_eq!(mode(&path), 0o600);
    assert_eq!(
        mode(&leftover),
        0o644,
        "someone else's file is left alone, not adopted"
    );
}

#[test]
fn rewrite_keeps_other_secrets() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("secrets.json");
    let store = JsonSecretStore::new(&path);
    store.set("a", "1").unwrap();
    store.set("b", "2").unwrap();
    store.set("a", "updated").unwrap();
    let saved: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(saved, serde_json::json!({"a": "updated", "b": "2"}));
}

#[test]
fn reads_a_file_the_python_release_wrote() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("secrets.json");
    fs::write(&path, "{\n  \"OpenAI\": \"sk-py\"\n}\n").unwrap();
    assert_eq!(
        JsonSecretStore::new(&path)
            .get("OpenAI")
            .unwrap()
            .as_deref(),
        Some("sk-py")
    );
}

#[test]
fn corrupt_or_wrong_shaped_json_fails_loudly_and_is_not_overwritten() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("secrets.json");
    for (content, expect) in [("{not json", "not valid JSON"), ("[1]", "JSON object")] {
        fs::write(&path, content).unwrap();
        let err = JsonSecretStore::new(&path)
            .set("name", "value")
            .unwrap_err();
        assert!(err.to_string().contains(expect), "{content}: {err}");
        assert_eq!(fs::read_to_string(&path).unwrap(), content);
    }
}

#[test]
fn many_writers_at_once_never_lose_a_key() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config/secrets.json");
    let writers: Vec<_> = (0..16)
        .map(|i| {
            let path = path.clone();
            std::thread::spawn(move || {
                JsonSecretStore::new(path).set(&format!("key{i}"), &format!("secret{i}"))
            })
        })
        .collect();
    for writer in writers {
        writer.join().unwrap().unwrap();
    }
    let store = JsonSecretStore::new(&path);
    for i in 0..16 {
        let expected = format!("secret{i}");
        assert_eq!(
            store.get(&format!("key{i}")).unwrap().as_deref(),
            Some(expected.as_str())
        );
    }
    assert_eq!(mode(&path), 0o600);
}

#[test]
fn a_symlinked_secrets_file_stays_a_symlink_and_the_target_is_updated() {
    let dir = tempfile::tempdir().unwrap();
    let real_dir = dir.path().join("dotfiles");
    let config_dir = dir.path().join("config");
    fs::create_dir_all(&real_dir).unwrap();
    fs::create_dir_all(&config_dir).unwrap();
    let real = real_dir.join("secrets.json");
    fs::write(&real, "{\"a\": \"1\"}").unwrap();
    let link = config_dir.join("secrets.json");
    std::os::unix::fs::symlink(&real, &link).unwrap();

    JsonSecretStore::new(&link).set("b", "2").unwrap();

    assert!(fs::symlink_metadata(&link)
        .unwrap()
        .file_type()
        .is_symlink());
    let saved: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&real).unwrap()).unwrap();
    assert_eq!(saved, serde_json::json!({"a": "1", "b": "2"}));
}

#[test]
fn reading_tightens_a_secrets_file_that_others_can_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("secrets.json");
    fs::write(&path, "{\"a\": \"1\"}").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(
        JsonSecretStore::new(&path).get("a").unwrap().as_deref(),
        Some("1")
    );
    assert_eq!(mode(&path), 0o600);
}

#[test]
fn an_existing_folder_that_others_can_read_is_tightened_before_writing() {
    let dir = tempfile::tempdir().unwrap();
    let folder = dir.path().join("config");
    fs::create_dir(&folder).unwrap();
    fs::set_permissions(&folder, fs::Permissions::from_mode(0o755)).unwrap();
    JsonSecretStore::new(folder.join("secrets.json"))
        .set("a", "1")
        .unwrap();
    assert_eq!(mode(&folder), 0o700);
}
