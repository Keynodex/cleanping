use cleanping_core::domain::models::Credential;

use super::*;
use crate::setup::testing::{temp_services, FakeTools, ScriptedConsole};

fn credential() -> Credential {
    Credential {
        id: Some(1),
        name: "DeepSeek".into(),
        api_url: "https://api.deepseek.com/v1/chat/completions".into(),
        model: "deepseek-chat".into(),
    }
}

#[test]
fn a_yes_runs_the_test_and_reports_success() {
    let (_dir, services) = temp_services();
    let tools = FakeTools::default();
    let mut console = ScriptedConsole::new(&["y"]);
    let result = run(&mut console, &tools, &services, &credential()).unwrap();
    assert_eq!(result, Tested::Passed);
    assert_eq!(*tools.tested.borrow(), ["DeepSeek"]);
    assert!(
        console.said_text().contains("OK:"),
        "{}",
        console.said_text()
    );
}

#[test]
fn enter_alone_also_runs_it() {
    let (_dir, services) = temp_services();
    let tools = FakeTools::default();
    let mut console = ScriptedConsole::new(&[""]);
    assert_eq!(
        run(&mut console, &tools, &services, &credential()).unwrap(),
        Tested::Passed
    );
}

#[test]
fn a_no_sends_nothing_and_says_how_to_test_later() {
    let (_dir, services) = temp_services();
    let tools = FakeTools::default();
    let mut console = ScriptedConsole::new(&["n"]);
    let result = run(&mut console, &tools, &services, &credential()).unwrap();
    assert_eq!(result, Tested::Skipped);
    assert!(tools.tested.borrow().is_empty());
    assert!(
        console.said_text().contains("cleanping keys test"),
        "{}",
        console.said_text()
    );
}

#[test]
fn a_failure_is_shown_as_it_is_and_is_not_an_error_of_the_setup() {
    let (_dir, services) = temp_services();
    let tools = FakeTools {
        test_failure: Some(
            "The test with \u{201c}DeepSeek\u{201d} failed: API returned HTTP 401.".into(),
        ),
        ..FakeTools::default()
    };
    let mut console = ScriptedConsole::new(&["y"]);
    let result = run(&mut console, &tools, &services, &credential()).unwrap();
    assert_eq!(result, Tested::Failed);
    assert!(
        console.said_text().contains("HTTP 401"),
        "{}",
        console.said_text()
    );
}

#[test]
fn the_question_says_only_fixed_words_are_sent() {
    let (_dir, services) = temp_services();
    let mut console = ScriptedConsole::new(&["n"]);
    run(
        &mut console,
        &FakeTools::default(),
        &services,
        &credential(),
    )
    .unwrap();
    let said = format!("{} {:?}", console.said_text(), console.questions);
    assert!(said.contains("never your text"), "{said}");
}
