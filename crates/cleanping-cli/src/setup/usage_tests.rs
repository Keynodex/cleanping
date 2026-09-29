use super::*;

fn text(shell: Option<Shell>) -> String {
    usage_lines(shell).join("\n")
}

#[test]
fn the_shell_is_read_from_the_shell_variable() {
    assert_eq!(shell_from_path("/usr/bin/zsh"), Some(Shell::Zsh));
    assert_eq!(shell_from_path("/bin/bash"), Some(Shell::Bash));
    assert_eq!(shell_from_path("zsh"), Some(Shell::Zsh));
    assert_eq!(shell_from_path("/usr/bin/fish"), None);
    assert_eq!(shell_from_path(""), None);
}

#[test]
fn the_editor_line_for_claude_code_and_codex_is_always_shown() {
    for shell in [Some(Shell::Zsh), Some(Shell::Bash), None] {
        let text = text(shell);
        assert!(text.contains("export VISUAL=\"cleanping edit\""), "{text}");
        assert!(text.contains("Ctrl+G"), "{text}");
        assert!(
            text.contains("Claude Code") && text.contains("Codex"),
            "{text}"
        );
    }
}

#[test]
fn zsh_gets_its_own_profile_and_init_line() {
    let text = text(Some(Shell::Zsh));
    assert!(text.contains("~/.zshrc"), "{text}");
    assert!(text.contains("eval \"$(cleanping init zsh)\""), "{text}");
    assert!(
        !text.contains("init bash") && !text.contains("~/.bashrc"),
        "{text}"
    );
}

#[test]
fn bash_gets_its_own_profile_and_init_line() {
    let text = text(Some(Shell::Bash));
    assert!(text.contains("~/.bashrc"), "{text}");
    assert!(text.contains("eval \"$(cleanping init bash)\""), "{text}");
    assert!(
        !text.contains("init zsh") && !text.contains("~/.zshrc"),
        "{text}"
    );
}

#[test]
fn an_unknown_shell_sees_both_choices() {
    let text = text(None);
    assert!(
        text.contains("init zsh") && text.contains("init bash"),
        "{text}"
    );
}

#[test]
fn it_says_nothing_was_changed_and_how_to_come_back() {
    let text = text(Some(Shell::Zsh));
    assert!(text.contains("did not change any of your files"), "{text}");
    assert!(text.contains("cleanping setup"), "{text}");
    assert!(text.contains("Ctrl-X Ctrl-P"), "{text}");
}
