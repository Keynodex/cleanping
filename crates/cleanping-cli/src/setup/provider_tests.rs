use cleanping_core::application::ports::SecretStore;
use cleanping_core::domain::local_server::LocalServer;
use cleanping_core::domain::models::CredentialInput;
use cleanping_core::domain::providers::PROVIDER_PRESETS;

use super::*;
use crate::setup::testing::{temp_services, FakeTools, ScriptedConsole};

const KEY: &str = "test-key-not-real";

/// The number that picks this provider in the list.
fn number(label: &str) -> String {
    let index = PROVIDER_PRESETS
        .iter()
        .position(|p| p.label == label)
        .unwrap();
    (index + 1).to_string()
}

fn ollama_with(models: &[&str]) -> FakeTools {
    FakeTools {
        server: Some(LocalServer::Running {
            models: models.iter().map(|m| (*m).to_string()).collect(),
        }),
        ..FakeTools::default()
    }
}

fn stored_key(services: &Services, name: &str) -> Option<String> {
    services.secrets().get(name).unwrap()
}

#[test]
fn a_remote_provider_is_saved_with_a_hidden_key_and_selected() {
    let (_dir, services) = temp_services();
    let mut console = ScriptedConsole::new(&[&number("DeepSeek")]).with_secrets(&[KEY]);
    let saved = run(&mut console, &FakeTools::default(), &services).unwrap();
    assert_eq!(saved.name, "DeepSeek");
    assert_eq!(
        saved.api_url,
        "https://api.deepseek.com/v1/chat/completions"
    );
    assert_eq!(saved.model, "deepseek-chat");
    assert_eq!(stored_key(&services, "DeepSeek").as_deref(), Some(KEY));
    assert_eq!(services.state.selected_credential_id().unwrap(), saved.id);
    assert!(
        !console.said_text().contains(KEY),
        "the key must never be printed"
    );
    assert!(
        console.questions.iter().any(|q| q.contains("hidden")),
        "{:?}",
        console.questions
    );
}

#[test]
fn a_blank_key_is_asked_for_again_and_too_many_stop_the_setup() {
    let (_dir, services) = temp_services();
    let mut console = ScriptedConsole::new(&[&number("OpenAI")]).with_secrets(&["", "  ", KEY]);
    run(&mut console, &FakeTools::default(), &services).unwrap();
    assert_eq!(stored_key(&services, "OpenAI").as_deref(), Some(KEY));

    let (_dir, services) = temp_services();
    let mut hopeless =
        ScriptedConsole::new(&[&number("OpenAI")]).with_secrets(&["", "", "", "", "", KEY]);
    assert!(run(&mut hopeless, &FakeTools::default(), &services).is_err());
    assert!(
        services.credentials.list().unwrap().is_empty(),
        "nothing half-saved"
    );
}

#[test]
fn a_saved_key_can_be_kept() {
    let (_dir, services) = temp_services();
    services
        .credentials
        .save(CredentialInput {
            name: "DeepSeek".into(),
            api_url: "https://api.deepseek.com/v1/chat/completions".into(),
            model: "deepseek-chat".into(),
            api_key: "old-key".into(),
        })
        .unwrap();
    let mut console = ScriptedConsole::new(&[&number("DeepSeek"), "y"]);
    run(&mut console, &FakeTools::default(), &services).unwrap();
    assert_eq!(
        stored_key(&services, "DeepSeek").as_deref(),
        Some("old-key")
    );
}

#[test]
fn declining_to_keep_the_saved_key_asks_for_a_new_one() {
    let (_dir, services) = temp_services();
    services
        .credentials
        .save(CredentialInput {
            name: "DeepSeek".into(),
            api_url: "https://api.deepseek.com/v1/chat/completions".into(),
            model: "deepseek-chat".into(),
            api_key: "old-key".into(),
        })
        .unwrap();
    let mut console = ScriptedConsole::new(&[&number("DeepSeek"), "n"]).with_secrets(&["new-key"]);
    run(&mut console, &FakeTools::default(), &services).unwrap();
    assert_eq!(
        stored_key(&services, "DeepSeek").as_deref(),
        Some("new-key")
    );
}

#[test]
fn a_local_model_needs_no_key_and_is_reported_ready() {
    let (_dir, services) = temp_services();
    let mut console = ScriptedConsole::new(&[&number("Ollama (local)")]);
    let tools = ollama_with(&["qwen2.5:7b"]);
    let saved = run(&mut console, &tools, &services).unwrap();
    assert_eq!(saved.name, "Ollama (local)");
    assert!(
        console.said_text().contains("Ollama is running and has"),
        "{}",
        console.said_text()
    );
    assert!(tools.pulled.borrow().is_empty());
    assert_eq!(console.left_over(), 0);
}

#[test]
fn a_missing_model_is_downloaded_only_after_a_yes() {
    let (_dir, services) = temp_services();
    let mut yes = ScriptedConsole::new(&[&number("Ollama (local)"), "y"]);
    let tools = ollama_with(&["llama3:8b"]);
    run(&mut yes, &tools, &services).unwrap();
    assert_eq!(*tools.pulled.borrow(), ["qwen2.5:7b"]);

    let (_dir, services) = temp_services();
    let mut no = ScriptedConsole::new(&[&number("Ollama (local)"), "n"]);
    let tools = ollama_with(&["llama3:8b"]);
    run(&mut no, &tools, &services).unwrap();
    assert!(tools.pulled.borrow().is_empty());
    assert!(
        no.said_text().contains("ollama pull qwen2.5:7b"),
        "{}",
        no.said_text()
    );
}

#[test]
fn enter_alone_does_not_start_a_download() {
    let (_dir, services) = temp_services();
    let mut console = ScriptedConsole::new(&[&number("Ollama (local)"), ""]);
    let tools = ollama_with(&[]);
    run(&mut console, &tools, &services).unwrap();
    assert!(tools.pulled.borrow().is_empty());
}

#[test]
fn a_download_that_fails_is_reported() {
    let (_dir, services) = temp_services();
    let mut console = ScriptedConsole::new(&[&number("Ollama (local)"), "y"]);
    let tools = FakeTools {
        pull_works: false,
        ..ollama_with(&[])
    };
    run(&mut console, &tools, &services).unwrap();
    assert!(
        console.said_text().contains("did not finish"),
        "{}",
        console.said_text()
    );
}

#[test]
fn ollama_that_is_not_installed_points_to_the_official_page_and_asks_nothing() {
    let (_dir, services) = temp_services();
    let mut console = ScriptedConsole::new(&[&number("Ollama (local)")]);
    let tools = FakeTools {
        server: Some(LocalServer::NotInstalled),
        ..FakeTools::default()
    };
    run(&mut console, &tools, &services).unwrap();
    assert!(
        console.said_text().contains("https://ollama.com/download"),
        "{}",
        console.said_text()
    );
    assert_eq!(
        console.questions.len(),
        1,
        "only the provider question: {:?}",
        console.questions
    );
    assert!(tools.pulled.borrow().is_empty());
}

#[test]
fn ollama_that_is_not_running_says_how_to_start_it() {
    let (_dir, services) = temp_services();
    let mut console = ScriptedConsole::new(&[&number("Ollama (local)")]);
    let tools = FakeTools {
        server: Some(LocalServer::NotRunning),
        ..FakeTools::default()
    };
    run(&mut console, &tools, &services).unwrap();
    assert!(
        console.said_text().contains("ollama serve"),
        "{}",
        console.said_text()
    );
}

#[test]
fn a_custom_provider_asks_for_name_address_and_model() {
    let (_dir, services) = temp_services();
    let mut console = ScriptedConsole::new(&[
        &number("Custom"),
        "My server",
        "https://llm.example.com/v1/chat/completions",
        "big-model",
    ])
    .with_secrets(&[KEY]);
    let saved = run(&mut console, &FakeTools::default(), &services).unwrap();
    assert_eq!(saved.name, "My server");
    assert_eq!(saved.api_url, "https://llm.example.com/v1/chat/completions");
    assert_eq!(saved.model, "big-model");
    assert_eq!(stored_key(&services, "My server").as_deref(), Some(KEY));
}

#[test]
fn a_custom_server_on_this_computer_needs_no_key() {
    let (_dir, services) = temp_services();
    let mut console = ScriptedConsole::new(&[
        &number("Custom"),
        "Local",
        "http://127.0.0.1:8080/v1/chat/completions",
        "m",
    ]);
    run(&mut console, &FakeTools::default(), &services).unwrap();
    assert_eq!(console.left_over(), 0);
}

#[test]
fn a_bad_custom_address_is_explained_and_everything_is_asked_again() {
    let (_dir, services) = temp_services();
    let mut console = ScriptedConsole::new(&[
        &number("Custom"),
        "Srv",
        "http://example.com/v1/chat/completions",
        "m",
        "Srv",
        "https://ok.example.com/v1/chat/completions",
        "m",
    ])
    .with_secrets(&[KEY]);
    let saved = run(&mut console, &FakeTools::default(), &services).unwrap();
    assert_eq!(saved.api_url, "https://ok.example.com/v1/chat/completions");
    assert!(
        console.said_text().contains("HTTPS"),
        "{}",
        console.said_text()
    );
}

#[test]
fn a_model_name_that_looks_like_an_option_is_never_handed_to_ollama() {
    let (_dir, services) = temp_services();
    let mut console = ScriptedConsole::new(&[
        &number("Custom"),
        "Local",
        "http://127.0.0.1:8080/v1/chat/completions",
        "--help",
        "y",
    ]);
    let tools = ollama_with(&[]);
    run(&mut console, &tools, &services).unwrap();
    assert!(tools.pulled.borrow().is_empty());
    assert!(
        console.said_text().contains("looks like an option"),
        "{}",
        console.said_text()
    );
}
