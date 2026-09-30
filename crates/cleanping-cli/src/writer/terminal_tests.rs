use super::*;
use crate::exit::code_for;

#[test]
fn two_terminals_are_fine() {
    assert_eq!(require(true, true), Ok(()));
}

#[test]
fn a_missing_terminal_on_either_side_is_bad_usage_with_a_clear_message() {
    for (stdin, stdout) in [(false, true), (true, false), (false, false)] {
        let error = require(stdin, stdout).unwrap_err();
        assert_eq!(code_for(&error), 2, "stdin {stdin}, stdout {stdout}");
        let message = error.to_string();
        assert!(message.contains("needs a terminal"), "{message}");
        assert!(!message.contains('\n'), "one line: {message}");
    }
}
