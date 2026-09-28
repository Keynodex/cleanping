use super::*;
use crate::application::test_support::FakePrompts;

#[test]
fn current_falls_back_to_the_default() {
    let svc = PromptService::new(FakePrompts::default());
    assert_eq!(svc.current().unwrap(), DEFAULT_INSTRUCTIONS);
    assert!(DEFAULT_INSTRUCTIONS.starts_with("You are a precise copy editor"));
}

#[test]
fn save_trims_and_current_returns_the_latest() {
    let svc = PromptService::new(FakePrompts::default());
    svc.save("  first  ").unwrap();
    svc.save("second").unwrap();
    assert_eq!(svc.current().unwrap(), "second");
    assert_eq!(svc.prompts.0.borrow()[0], "first");
}

#[test]
fn empty_save_is_rejected() {
    let svc = PromptService::new(FakePrompts::default());
    assert!(svc.save("   ").is_err());
    assert!(svc.prompts.0.borrow().is_empty());
}
