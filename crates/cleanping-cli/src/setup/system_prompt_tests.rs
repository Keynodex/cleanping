use cleanping_core::domain::prompt_presets::{preset_named, PROMPT_PRESETS};

use super::*;
use crate::setup::testing::{temp_services, ScriptedConsole};

fn number(name: &str) -> String {
    let index = PROMPT_PRESETS.iter().position(|p| p.name == name).unwrap();
    (index + 1).to_string()
}

fn body(name: &str) -> &'static str {
    preset_named(name).unwrap().body
}

#[test]
fn enter_keeps_the_current_prompt() {
    let (_dir, services) = temp_services();
    let mut console = ScriptedConsole::new(&[""]);
    run(&mut console, &services).unwrap();
    assert_eq!(services.prompts.current().unwrap(), body("default"));
    assert!(
        console.said_text().contains("Keeping"),
        "{}",
        console.said_text()
    );
}

#[test]
fn a_ready_made_prompt_can_be_chosen() {
    let (_dir, services) = temp_services();
    let mut console = ScriptedConsole::new(&[&number("concise")]);
    run(&mut console, &services).unwrap();
    assert_eq!(services.prompts.current().unwrap(), body("concise"));
    assert!(
        console.said_text().contains("concise"),
        "{}",
        console.said_text()
    );
}

#[test]
fn every_ready_made_prompt_is_listed_and_the_one_in_use_is_marked() {
    let (_dir, services) = temp_services();
    services.prompts.save(body("typos")).unwrap();
    let mut console = ScriptedConsole::new(&[""]);
    run(&mut console, &services).unwrap();
    let said = console.said_text();
    for preset in PROMPT_PRESETS {
        assert!(said.contains(preset.name), "{said}");
    }
    let marked: Vec<&str> = said.lines().filter(|l| l.contains("(now)")).collect();
    assert_eq!(marked.len(), 1, "{said}");
    assert!(marked[0].contains("typos"), "{said}");
}

#[test]
fn your_own_prompt_is_kept_by_default_and_marked() {
    let (_dir, services) = temp_services();
    services.prompts.save("Answer like a pirate.").unwrap();
    let mut console = ScriptedConsole::new(&[""]);
    run(&mut console, &services).unwrap();
    assert_eq!(services.prompts.current().unwrap(), "Answer like a pirate.");
    assert_eq!(
        console.questions.len(),
        1,
        "no extra question: {:?}",
        console.questions
    );
    let said = console.said_text();
    assert!(
        said.lines()
            .any(|l| l.contains("Keep my own prompt") && l.contains("(now)")),
        "{said}"
    );
}

#[test]
fn replacing_your_own_prompt_needs_a_yes() {
    let (_dir, services) = temp_services();
    services.prompts.save("Answer like a pirate.").unwrap();
    let mut no = ScriptedConsole::new(&[&number("typos"), "n"]);
    run(&mut no, &services).unwrap();
    assert_eq!(services.prompts.current().unwrap(), "Answer like a pirate.");

    let mut yes = ScriptedConsole::new(&[&number("typos"), "y"]);
    run(&mut yes, &services).unwrap();
    assert_eq!(services.prompts.current().unwrap(), body("typos"));
}

#[test]
fn it_says_how_to_write_your_own() {
    let (_dir, services) = temp_services();
    let mut console = ScriptedConsole::new(&[""]);
    run(&mut console, &services).unwrap();
    assert!(
        console.said_text().contains("cleanping prompt set"),
        "{}",
        console.said_text()
    );
}

#[test]
fn the_structure_prompt_is_a_numbered_choice_after_friendly() {
    let (_dir, services) = temp_services();
    let after_friendly = (number("friendly").parse::<usize>().unwrap() + 1).to_string();
    let mut console = ScriptedConsole::new(&[&after_friendly]);
    run(&mut console, &services).unwrap();
    let said = console.said_text();
    let line = format!("  {after_friendly}) structure Fix and lay out");
    assert!(said.contains(&line), "no structure preset in: {said}");
    assert_eq!(services.prompts.current().unwrap(), body("structure"));
    assert!(said.contains("\u{201c}structure\u{201d} preset"), "{said}");
}
