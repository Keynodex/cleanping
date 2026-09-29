use super::*;

fn models(names: &[&str]) -> Vec<String> {
    names.iter().map(|n| (*n).to_string()).collect()
}

#[test]
fn a_model_is_present_by_exact_name() {
    assert!(model_present(
        &models(&["qwen2.5:7b", "llama3:8b"]),
        "qwen2.5:7b"
    ));
    assert!(!model_present(&models(&["qwen2.5:7b"]), "qwen2.5:14b"));
    assert!(!model_present(&[], "qwen2.5:7b"));
}

#[test]
fn a_name_without_a_tag_means_latest() {
    assert!(model_present(&models(&["llama3:latest"]), "llama3"));
    assert!(!model_present(&models(&["llama3:8b"]), "llama3"));
}

#[test]
fn only_the_standard_ollama_address_counts_as_the_default() {
    assert!(is_default_ollama_url(
        "http://127.0.0.1:11434/v1/chat/completions"
    ));
    assert!(is_default_ollama_url(
        "http://localhost:11434/v1/chat/completions"
    ));
    assert!(!is_default_ollama_url(
        "http://127.0.0.1:8080/v1/chat/completions"
    ));
    assert!(!is_default_ollama_url(
        "https://api.openai.com/v1/chat/completions"
    ));
    assert!(!is_default_ollama_url(
        "http://example.com:11434/v1/chat/completions"
    ));
    assert!(!is_default_ollama_url("not a url"));
}

#[test]
fn a_running_server_missing_the_model_needs_a_pull() {
    let server = LocalServer::Running {
        models: models(&["llama3:8b"]),
    };
    assert_eq!(
        diagnose(&server, "qwen2.5:7b", false),
        Some(LocalFix::PullModel)
    );
    assert_eq!(
        diagnose(&server, "qwen2.5:7b", true),
        Some(LocalFix::PullModel)
    );
}

#[test]
fn a_running_server_with_the_model_has_nothing_to_fix() {
    let server = LocalServer::Running {
        models: models(&["qwen2.5:7b"]),
    };
    assert_eq!(diagnose(&server, "qwen2.5:7b", true), None);
}

#[test]
fn install_and_start_advice_is_only_for_the_default_address() {
    // On another port we cannot know it was meant to be Ollama, so we say nothing.
    assert_eq!(
        diagnose(&LocalServer::NotInstalled, "m", true),
        Some(LocalFix::Install)
    );
    assert_eq!(
        diagnose(&LocalServer::NotRunning, "m", true),
        Some(LocalFix::Start)
    );
    assert_eq!(diagnose(&LocalServer::NotInstalled, "m", false), None);
    assert_eq!(diagnose(&LocalServer::NotRunning, "m", false), None);
}

#[test]
fn a_server_that_is_not_ollama_gets_no_ollama_advice() {
    assert_eq!(diagnose(&LocalServer::Other, "m", true), None);
}
