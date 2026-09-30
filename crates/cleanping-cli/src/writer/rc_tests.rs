use std::os::unix::fs::PermissionsExt;

use super::*;

fn mode(path: &Path) -> u32 {
    std::fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[test]
fn the_script_is_written_as_dot_zshrc_inside_a_private_folder() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("data/writer");
    install(&dir).unwrap();
    let rc = dir.join(".zshrc");
    assert_eq!(std::fs::read_to_string(&rc).unwrap(), super::super::SCRIPT);
    assert_eq!((mode(&dir), mode(&rc)), (0o700, 0o600));
}

#[test]
fn an_old_loose_file_and_folder_are_replaced_and_tightened() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("writer");
    std::fs::create_dir(&dir).unwrap();
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
    let rc = dir.join(".zshrc");
    std::fs::write(&rc, "old").unwrap();
    std::fs::set_permissions(&rc, std::fs::Permissions::from_mode(0o644)).unwrap();
    install(&dir).unwrap();
    assert_eq!(std::fs::read_to_string(&rc).unwrap(), super::super::SCRIPT);
    assert_eq!((mode(&dir), mode(&rc)), (0o700, 0o600));
}

#[test]
fn a_link_left_in_the_place_of_the_file_is_replaced_not_followed() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("writer");
    std::fs::create_dir(&dir).unwrap();
    let elsewhere = root.path().join("elsewhere");
    std::fs::write(&elsewhere, "keep").unwrap();
    std::os::unix::fs::symlink(&elsewhere, dir.join(".zshrc")).unwrap();
    install(&dir).unwrap();
    assert_eq!(std::fs::read_to_string(&elsewhere).unwrap(), "keep");
    assert_eq!(
        std::fs::read_to_string(dir.join(".zshrc")).unwrap(),
        super::super::SCRIPT
    );
}

#[test]
fn nothing_but_the_startup_file_is_left_behind() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("writer");
    install(&dir).unwrap();
    install(&dir).unwrap();
    let names: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(names, vec![std::ffi::OsString::from(".zshrc")]);
}
