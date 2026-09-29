use cleanping_core::domain::local_server::LocalServer;
use cleanping_core::domain::models::CredentialInput;
use cleanping_core::domain::prompt_presets::{preset_named, PROMPT_PRESETS};
use cleanping_core::domain::providers::PROVIDER_PRESETS;

use super::*;
use crate::setup::testing::{temp_services, FakeTools, ScriptedConsole};

const KEY: &str = "test-key-not-real";
const QUIT: &str = "5";

fn provider_number(label: &str) -> String {
    (PROVIDER_PRESETS
        .iter()
        .position(|p| p.label == label)
        .unwrap()
        + 1)
    .to_string()
}

fn prompt_number(name: &str) -> String {
    (PROMPT_PRESETS.iter().position(|p| p.name == name).unwrap() + 1).to_string()
}

fn save_deepseek(services: &Services) {
    let saved = services
        .credentials
        .save(CredentialInput {
            name: "DeepSeek".into(),
            api_url: "https://api.deepseek.com/v1/chat/completions".into(),
            model: "deepseek-chat".into(),
            api_key: KEY.into(),
        })
        .unwrap();
    services.state.select_credential(saved.id).unwrap();
}

fn selected_name(services: &Services) -> String {
    let selected = services.state.selected_credential_id().unwrap();
    services.credentials.resolve(None, selected).unwrap().name
}

#[test]
fn a_first_run_goes_through_the_steps_and_ends_with_how_to_use_it() {
    let (_dir, services) = temp_services();
    let tools = FakeTools::default();
    let mut console =
        ScriptedConsole::new(&[&provider_number("DeepSeek"), "", "y"]).with_secrets(&[KEY]);
    run(&mut console, &tools, &services, Some(Shell::Zsh)).unwrap();
    let said = console.said_text();
    assert!(said.contains("All set"), "{said}");
    assert!(said.contains("export VISUAL=\"cleanping edit\""), "{said}");
    assert!(said.contains("eval \"$(cleanping init zsh)\""), "{said}");
    assert_eq!(*tools.tested.borrow(), ["DeepSeek"]);
    assert_eq!(selected_name(&services), "DeepSeek");
    assert_eq!(console.left_over(), 0);
}

#[test]
fn a_failed_test_is_shown_and_the_setup_still_finishes() {
    let (_dir, services) = temp_services();
    let tools = FakeTools {
        test_failure: Some(
            "The test with \u{201c}DeepSeek\u{201d} failed: API returned HTTP 401.".into(),
        ),
        ..FakeTools::default()
    };
    let mut console =
        ScriptedConsole::new(&[&provider_number("DeepSeek"), "", "y"]).with_secrets(&[KEY]);
    run(&mut console, &tools, &services, Some(Shell::Bash)).unwrap();
    let said = console.said_text();
    assert!(said.contains("HTTP 401"), "{said}");
    assert!(said.contains("connection test failed"), "{said}");
    assert!(!said.contains("All set"), "{said}");
    assert!(said.contains("init bash"), "usage is still shown: {said}");
    assert_eq!(selected_name(&services), "DeepSeek", "the key stays saved");
}

#[test]
fn skipping_the_test_says_it_was_not_tested() {
    let (_dir, services) = temp_services();
    let mut console =
        ScriptedConsole::new(&[&provider_number("DeepSeek"), "", "n"]).with_secrets(&[KEY]);
    run(&mut console, &FakeTools::default(), &services, None).unwrap();
    let said = console.said_text();
    assert!(said.contains("not tested yet"), "{said}");
    assert!(!said.contains("All set"), "{said}");
}

#[test]
fn running_setup_again_shows_a_menu_instead_of_starting_over() {
    let (_dir, services) = temp_services();
    save_deepseek(&services);
    let mut console = ScriptedConsole::new(&[QUIT]);
    run(&mut console, &FakeTools::default(), &services, None).unwrap();
    let said = console.said_text();
    assert!(said.contains("What would you like to change?"), "{said}");
    assert!(
        said.contains("DeepSeek") && said.contains("deepseek-chat"),
        "{said}"
    );
    assert!(!said.contains("Step 1"), "{said}");
}

#[test]
fn enter_in_the_menu_quits() {
    let (_dir, services) = temp_services();
    save_deepseek(&services);
    let mut console = ScriptedConsole::new(&[""]);
    run(&mut console, &FakeTools::default(), &services, None).unwrap();
    assert_eq!(console.left_over(), 0);
}

#[test]
fn the_menu_can_change_the_system_prompt() {
    let (_dir, services) = temp_services();
    save_deepseek(&services);
    let mut console = ScriptedConsole::new(&["2", &prompt_number("concise"), QUIT]);
    run(&mut console, &FakeTools::default(), &services, None).unwrap();
    assert_eq!(
        services.prompts.current().unwrap(),
        preset_named("concise").unwrap().body
    );
}

#[test]
fn the_menu_can_test_the_connection() {
    let (_dir, services) = temp_services();
    save_deepseek(&services);
    let tools = FakeTools::default();
    let mut console = ScriptedConsole::new(&["3", "y", QUIT]);
    run(&mut console, &tools, &services, None).unwrap();
    assert_eq!(*tools.tested.borrow(), ["DeepSeek"]);
}

#[test]
fn the_menu_can_show_how_to_use_it() {
    let (_dir, services) = temp_services();
    save_deepseek(&services);
    let mut console = ScriptedConsole::new(&["4", QUIT]);
    run(
        &mut console,
        &FakeTools::default(),
        &services,
        Some(Shell::Zsh),
    )
    .unwrap();
    assert!(console
        .said_text()
        .contains("export VISUAL=\"cleanping edit\""));
}

#[test]
fn the_menu_can_switch_to_another_provider() {
    let (_dir, services) = temp_services();
    save_deepseek(&services);
    let tools = FakeTools {
        server: Some(LocalServer::Running {
            models: vec!["qwen2.5:7b".into()],
        }),
        ..FakeTools::default()
    };
    let mut console = ScriptedConsole::new(&["1", &provider_number("Ollama (local)"), QUIT]);
    run(&mut console, &tools, &services, None).unwrap();
    assert_eq!(selected_name(&services), "Ollama (local)");
}

#[test]
fn testing_from_the_menu_with_no_key_chosen_asks_to_choose_one_first() {
    let (_dir, services) = temp_services();
    for name in ["One", "Two"] {
        services
            .credentials
            .save(CredentialInput {
                name: name.into(),
                api_url: "http://127.0.0.1:9/v1/chat/completions".into(),
                model: "m".into(),
                api_key: String::new(),
            })
            .unwrap();
    }
    let tools = FakeTools::default();
    let mut console = ScriptedConsole::new(&["3", QUIT]);
    run(&mut console, &tools, &services, None).unwrap();
    let said = console.said_text();
    assert!(said.contains("none selected"), "{said}");
    assert!(said.contains("Choose your AI first"), "{said}");
    assert!(tools.tested.borrow().is_empty());
}
