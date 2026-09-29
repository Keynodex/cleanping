use super::*;
use crate::setup::testing::ScriptedConsole;

fn options() -> Vec<String> {
    vec!["one".into(), "two".into(), "three".into()]
}

#[test]
fn choose_returns_the_zero_based_index_of_the_number_typed() {
    let mut console = ScriptedConsole::new(&["2"]);
    assert_eq!(choose(&mut console, "Pick", &options(), None).unwrap(), 1);
}

#[test]
fn choose_lists_every_option_numbered_from_one() {
    let mut console = ScriptedConsole::new(&["1"]);
    choose(&mut console, "Pick one:", &options(), None).unwrap();
    let said = console.said_text();
    assert!(said.contains("Pick one:"), "{said}");
    for line in ["1) one", "2) two", "3) three"] {
        assert!(said.contains(line), "{said}");
    }
}

#[test]
fn choose_ignores_blanks_around_the_number() {
    let mut console = ScriptedConsole::new(&["  3 "]);
    assert_eq!(choose(&mut console, "Pick", &options(), None).unwrap(), 2);
}

#[test]
fn choose_asks_again_after_something_that_is_not_a_choice() {
    let mut console = ScriptedConsole::new(&["x", "0", "9", "-1", "1"]);
    assert_eq!(choose(&mut console, "Pick", &options(), None).unwrap(), 0);
    assert!(console.said_text().contains("from 1 to 3"));
}

#[test]
fn enter_takes_the_default_when_there_is_one() {
    let mut console = ScriptedConsole::new(&[""]);
    assert_eq!(
        choose(&mut console, "Pick", &options(), Some(2)).unwrap(),
        2
    );
    assert!(
        console.questions[0].contains("Enter = 3"),
        "{:?}",
        console.questions
    );
}

#[test]
fn enter_alone_is_not_an_answer_without_a_default() {
    let mut console = ScriptedConsole::new(&["", "2"]);
    assert_eq!(choose(&mut console, "Pick", &options(), None).unwrap(), 1);
}

#[test]
fn choose_gives_up_after_too_many_bad_answers() {
    let mut console = ScriptedConsole::new(&["a", "b", "c", "d", "e", "1"]);
    let error = choose(&mut console, "Pick", &options(), None).unwrap_err();
    assert!(error.to_string().contains("Setup stopped"), "{error}");
}

#[test]
fn choose_stops_when_the_input_ends() {
    let mut console = ScriptedConsole::new(&[]);
    assert!(choose(&mut console, "Pick", &options(), None).is_err());
}

#[test]
fn confirm_understands_yes_and_no_in_any_case() {
    for (answer, expected) in [
        ("y", true),
        ("YES", true),
        ("Yes", true),
        ("n", false),
        ("No", false),
        ("NO", false),
    ] {
        let mut console = ScriptedConsole::new(&[answer]);
        assert_eq!(
            confirm(&mut console, "Sure?", true).unwrap(),
            expected,
            "{answer}"
        );
    }
}

#[test]
fn enter_takes_the_default_answer_and_the_question_shows_it() {
    let mut yes = ScriptedConsole::new(&[""]);
    assert!(confirm(&mut yes, "Sure?", true).unwrap());
    assert!(yes.questions[0].contains("[Y/n]"), "{:?}", yes.questions);
    let mut no = ScriptedConsole::new(&[""]);
    assert!(!confirm(&mut no, "Sure?", false).unwrap());
    assert!(no.questions[0].contains("[y/N]"), "{:?}", no.questions);
}

#[test]
fn confirm_asks_again_after_an_unclear_answer_and_then_gives_up() {
    let mut console = ScriptedConsole::new(&["maybe", "y"]);
    assert!(confirm(&mut console, "Sure?", false).unwrap());
    let mut hopeless = ScriptedConsole::new(&["a", "b", "c", "d", "e", "y"]);
    assert!(confirm(&mut hopeless, "Sure?", false).is_err());
}
