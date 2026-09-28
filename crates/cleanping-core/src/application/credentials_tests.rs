use super::*;
use crate::application::test_support::{FakeCredentials, FakeSecrets};
use crate::domain::errors::CleanpingError;
use crate::domain::models::Credential;

fn input(key: &str, name: &str) -> CredentialInput {
    CredentialInput {
        name: name.into(),
        api_url: "https://api.openai.com/v1/chat/completions".into(),
        model: "gpt-4o-mini".into(),
        api_key: key.into(),
    }
}

fn service(secrets: FakeSecrets) -> CredentialService<FakeCredentials, FakeSecrets> {
    CredentialService::new(FakeCredentials::default(), secrets)
}

#[test]
fn save_new_stores_the_secret_and_the_row() {
    let svc = service(FakeSecrets::default());
    let saved = svc.save(input("sk-x", "OpenAI")).unwrap();
    assert_eq!(svc.secrets.get("OpenAI").unwrap().as_deref(), Some("sk-x"));
    assert_eq!(svc.get(saved.id.unwrap()).unwrap().name, "OpenAI");
}

#[test]
fn save_existing_with_blank_key_keeps_the_secret() {
    let svc = service(FakeSecrets::default());
    svc.save(input("sk-old", "OpenAI")).unwrap();
    let renamed_model = CredentialInput {
        model: "newer-model".into(),
        ..input("", "OpenAI")
    };
    svc.save(renamed_model).unwrap();
    assert_eq!(svc.list().unwrap()[0].model, "newer-model");
    assert_eq!(
        svc.secrets.get("OpenAI").unwrap().as_deref(),
        Some("sk-old")
    );
}

#[test]
fn save_new_with_blank_key_is_rejected() {
    let svc = service(FakeSecrets::default());
    let err = svc.save(input("", "OpenAI")).unwrap_err();
    assert_eq!(err.to_string(), "Enter an API key.");
    assert!(svc.list().unwrap().is_empty());
}

#[test]
fn save_new_local_credential_needs_no_secret() {
    let svc = service(FakeSecrets::default());
    let local = CredentialInput {
        api_url: "http://127.0.0.1:11434/v1/chat/completions".into(),
        ..input("", "Ollama")
    };
    assert!(svc.save(local).is_ok());
    assert_eq!(svc.secrets.get("Ollama").unwrap(), None);
}

#[test]
fn invalid_fields_are_rejected_before_anything_is_stored() {
    let svc = service(FakeSecrets::default());
    let bad = CredentialInput {
        api_url: "http://remote.example.com/v1".into(),
        ..input("sk-x", "Remote")
    };
    assert!(svc.save(bad).is_err());
    assert_eq!(svc.secrets.get("Remote").unwrap(), None);
}

#[test]
fn delete_removes_the_row_and_the_secret() {
    let svc = service(FakeSecrets::default());
    let saved = svc.save(input("sk-x", "OpenAI")).unwrap();
    svc.delete(saved.id.unwrap()).unwrap();
    assert!(svc.list().unwrap().is_empty());
    assert_eq!(svc.secrets.get("OpenAI").unwrap(), None);
}

fn seed(svc: &CredentialService<FakeCredentials, FakeSecrets>, names: &[&str]) {
    for (index, name) in names.iter().enumerate() {
        svc.credentials.items.borrow_mut().push(Credential {
            id: Some(index as i64 + 1),
            name: (*name).into(),
            api_url: "https://api.example.com/v1".into(),
            model: "m".into(),
        });
    }
}

#[test]
fn find_by_name_prefers_the_exact_spelling() {
    let svc = service(FakeSecrets::default());
    seed(&svc, &["Foo", "foo"]);
    assert_eq!(svc.find_by_name("foo").unwrap().id, Some(2));
    assert_eq!(svc.find_by_name("Foo").unwrap().id, Some(1));
}

#[test]
fn find_by_name_accepts_a_unique_case_insensitive_match() {
    let svc = service(FakeSecrets::default());
    seed(&svc, &["OpenAI"]);
    assert_eq!(svc.find_by_name("openai").unwrap().name, "OpenAI");
}

#[test]
fn find_by_name_refuses_an_ambiguous_match_and_reports_a_missing_one() {
    let svc = service(FakeSecrets::default());
    seed(&svc, &["Foo", "foo"]);
    let ambiguous = svc.find_by_name("FOO").unwrap_err();
    assert!(matches!(ambiguous, CleanpingError::Validation(_)));
    assert!(ambiguous.to_string().contains("exact"));
    assert!(matches!(
        svc.find_by_name("nope"),
        Err(CleanpingError::NotFound(_))
    ));
}

#[test]
fn save_refuses_a_new_name_that_differs_only_by_case() {
    let svc = service(FakeSecrets::default());
    svc.save(input("sk-x", "OpenAI")).unwrap();
    let err = svc.save(input("sk-y", "openai")).unwrap_err();
    assert!(err.to_string().contains("only by case"), "{err}");
    assert_eq!(svc.list().unwrap().len(), 1);
}

#[test]
fn an_exact_name_can_still_be_updated_when_a_case_variant_also_exists() {
    let svc = service(FakeSecrets::default());
    seed(&svc, &["Foo", "foo"]);
    let updated = CredentialInput {
        model: "newer-model".into(),
        ..input("sk-new", "Foo")
    };
    let saved = svc.save(updated).unwrap();
    assert_eq!((saved.id, saved.model.as_str()), (Some(1), "newer-model"));
    assert_eq!(svc.list().unwrap().len(), 2);
    assert_eq!(svc.secrets.get("Foo").unwrap().as_deref(), Some("sk-new"));
    assert!(svc.save(input("sk-z", "FOO")).is_err());
}

#[test]
fn a_blank_key_never_adopts_a_secret_left_over_without_a_row() {
    let svc = service(FakeSecrets::with("Prov", "sk-old"));
    let elsewhere = CredentialInput {
        api_url: "https://evil.example/v1".into(),
        ..input("", "Prov")
    };
    let err = svc.save(elsewhere).unwrap_err();
    assert!(err.to_string().contains("enter the API key again"), "{err}");
    assert!(svc.list().unwrap().is_empty());
    assert_eq!(svc.secrets.get("Prov").unwrap().as_deref(), Some("sk-old"));
    let with_key = svc.save(input("sk-new", "Prov")).unwrap();
    assert_eq!(with_key.name, "Prov");
    assert_eq!(svc.secrets.get("Prov").unwrap().as_deref(), Some("sk-new"));
}

#[test]
fn a_blank_key_with_a_new_address_asks_for_the_key_again() {
    let svc = service(FakeSecrets::default());
    svc.save(input("sk-x", "OpenAI")).unwrap();
    let moved = CredentialInput {
        api_url: "https://evil.example/v1".into(),
        ..input("", "OpenAI")
    };
    let err = svc.save(moved).unwrap_err();
    assert!(err.to_string().contains("enter the API key again"), "{err}");
    assert_eq!(
        svc.list().unwrap()[0].api_url,
        "https://api.openai.com/v1/chat/completions"
    );
}

#[test]
fn a_blank_key_with_only_a_new_path_keeps_the_saved_key() {
    let svc = service(FakeSecrets::default());
    svc.save(input("sk-x", "OpenAI")).unwrap();
    let same_host = CredentialInput {
        api_url: "https://api.openai.com/v1/other".into(),
        ..input("", "OpenAI")
    };
    svc.save(same_host).unwrap();
    assert_eq!(svc.secrets.get("OpenAI").unwrap().as_deref(), Some("sk-x"));
}

#[test]
fn a_failed_secret_write_leaves_no_half_saved_key() {
    let svc = service(FakeSecrets::default());
    svc.secrets.failing.set(true);
    assert!(svc.save(input("sk-x", "OpenAI")).is_err());
    assert!(
        svc.list().unwrap().is_empty(),
        "the new row must be rolled back"
    );
}

#[test]
fn a_failed_row_write_leaves_no_orphan_secret() {
    let svc = service(FakeSecrets::default());
    svc.credentials.failing.set(true);
    assert!(svc.save(input("sk-x", "OpenAI")).is_err());
    assert_eq!(svc.secrets.get("OpenAI").unwrap(), None);
}

#[test]
fn a_failed_secret_delete_keeps_the_row_so_the_delete_can_be_retried() {
    let svc = service(FakeSecrets::default());
    let saved = svc.save(input("sk-x", "OpenAI")).unwrap();
    svc.secrets.failing.set(true);
    assert!(svc.delete(saved.id.unwrap()).is_err());
    assert_eq!(svc.list().unwrap().len(), 1);
    svc.secrets.failing.set(false);
    svc.delete(saved.id.unwrap()).unwrap();
    assert!(svc.list().unwrap().is_empty());
}

#[test]
fn resolve_prefers_the_named_key_then_the_selected_one() {
    let svc = service(FakeSecrets::default());
    seed(&svc, &["Alpha", "Beta"]);
    assert_eq!(svc.resolve(Some("beta"), Some(1)).unwrap().name, "Beta");
    assert_eq!(svc.resolve(None, Some(2)).unwrap().name, "Beta");
}

#[test]
fn resolve_falls_back_only_when_exactly_one_key_exists() {
    let svc = service(FakeSecrets::default());
    seed(&svc, &["Only"]);
    assert_eq!(svc.resolve(None, None).unwrap().name, "Only");
    assert_eq!(svc.resolve(None, Some(99)).unwrap().name, "Only");
}

#[test]
fn resolve_refuses_to_guess_between_several_keys_or_with_none() {
    let svc = service(FakeSecrets::default());
    assert!(matches!(
        svc.resolve(None, None),
        Err(CleanpingError::MissingCredential(_))
    ));
    seed(&svc, &["Alpha", "Beta"]);
    let err = svc.resolve(None, None).unwrap_err();
    assert!(matches!(err, CleanpingError::MissingCredential(_)));
    assert!(err.to_string().contains("none is selected"), "{err}");
    assert!(matches!(
        svc.resolve(None, Some(99)),
        Err(CleanpingError::MissingCredential(_))
    ));
}
