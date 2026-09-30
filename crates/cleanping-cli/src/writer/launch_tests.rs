use std::ffi::OsString;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

use super::*;
use crate::exit::code_for;

fn parts(path: &OsString) -> Vec<PathBuf> {
    std::env::split_paths(path).collect()
}

#[test]
fn the_binary_folder_goes_first_and_the_rest_keeps_its_order() {
    let existing = OsString::from("/usr/bin:/bin");
    let path = path_with_first(Path::new("/opt/cleanping/bin"), Some(&existing)).unwrap();
    let wanted: Vec<PathBuf> = ["/opt/cleanping/bin", "/usr/bin", "/bin"]
        .map(PathBuf::from)
        .to_vec();
    assert_eq!(parts(&path), wanted);
}

#[test]
fn with_no_path_at_all_the_binary_folder_is_the_whole_path() {
    let path = path_with_first(Path::new("/opt/cleanping/bin"), None).unwrap();
    assert_eq!(parts(&path), vec![PathBuf::from("/opt/cleanping/bin")]);
}

#[test]
fn a_folder_that_cannot_be_part_of_a_path_gives_none() {
    assert_eq!(path_with_first(Path::new("/odd:name"), None), None);
}

#[test]
fn a_missing_zsh_is_explained_and_is_a_failure_not_bad_input() {
    let error = start_error(&Error::from(ErrorKind::NotFound));
    let message = error.to_string();
    assert!(message.contains("needs zsh"), "{message}");
    assert_eq!(code_for(&error), 1);
}

#[test]
fn any_other_start_problem_is_a_failure_with_a_clear_message() {
    let error = start_error(&Error::from(ErrorKind::PermissionDenied));
    assert!(error.to_string().contains("Could not start zsh"), "{error}");
    assert_eq!(code_for(&error), 1);
}
