use std::fs;

use super::*;
use crate::input::MAX_INPUT_BYTES;

fn scratch(bytes: &[u8]) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("prompt.txt");
    fs::write(&path, bytes).unwrap();
    (dir, path)
}

#[test]
fn reads_the_text_as_it_is() {
    let (_dir, path) = scratch(b"hello  world\n");
    assert_eq!(read(&path).unwrap(), "hello  world\n");
}

#[test]
fn refuses_text_that_is_not_utf8() {
    let (_dir, path) = scratch(&[0x68, 0xff, 0xfe]);
    assert!(read(&path).unwrap_err().to_string().contains("UTF-8"));
}

#[test]
fn refuses_a_file_over_the_input_limit() {
    let (_dir, path) = scratch(&vec![b'a'; MAX_INPUT_BYTES + 1]);
    assert!(read(&path).unwrap_err().to_string().contains("too long"));
}

#[test]
fn accepts_a_file_exactly_at_the_limit() {
    let (_dir, path) = scratch(&vec![b'a'; MAX_INPUT_BYTES]);
    assert_eq!(read(&path).unwrap().len(), MAX_INPUT_BYTES);
}

#[test]
fn refuses_a_folder_or_a_missing_file() {
    let (dir, path) = scratch(b"x");
    assert!(read(dir.path()).is_err());
    fs::remove_file(&path).unwrap();
    assert!(read(&path).is_err());
}

#[test]
fn the_reply_keeps_the_trailing_newline_the_file_had() {
    assert_eq!(replacement("rough\n", "Clean."), "Clean.\n");
    assert_eq!(replacement("rough\r\n", "Clean."), "Clean.\r\n");
    assert_eq!(replacement("rough", "Clean."), "Clean.");
}

#[test]
fn the_reply_never_ends_with_more_than_one_newline() {
    assert_eq!(replacement("rough\n", "Clean.\n\n"), "Clean.\n");
    assert_eq!(replacement("rough", "Clean.\n"), "Clean.");
}

#[test]
fn writing_replaces_the_whole_file() {
    let (_dir, path) = scratch(b"a much longer original text");
    write(&path, "short").unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "short");
}
