//! The closing screen: how to use CleanPing where the user works. It only prints lines; it
//! never edits the user's shell profile.

use crate::args::Shell;

/// `zsh` or `bash` from a path such as the value of `$SHELL`.
pub fn shell_from_path(path: &str) -> Option<Shell> {
    match path.rsplit('/').next() {
        Some("zsh") => Some(Shell::Zsh),
        Some("bash") => Some(Shell::Bash),
        _ => None,
    }
}

pub fn usage_lines(shell: Option<Shell>) -> Vec<String> {
    let mut lines = vec![
        "How to use CleanPing".to_string(),
        String::new(),
        "  In Claude Code or Codex: add this line to your shell profile, open a new".into(),
        "  terminal, then press Ctrl+G in the app to fix your message:".into(),
        String::new(),
        "      export VISUAL=\"cleanping edit\"".into(),
        String::new(),
        "  At your shell prompt: add the line below, open a new terminal, type a rough".into(),
        "  command line and press Ctrl-X Ctrl-P (again to get your original back):".into(),
        String::new(),
    ];
    match shell {
        Some(Shell::Zsh) => lines.extend(init_line("~/.zshrc", "zsh")),
        Some(Shell::Bash) => lines.extend(init_line("~/.bashrc", "bash")),
        None => {
            lines.extend(init_line("~/.zshrc", "zsh"));
            lines.extend(init_line("~/.bashrc", "bash"));
        }
    }
    lines.extend([
        "  On the command line: cleanping \"plz fix teh login pgae\"".into(),
        String::new(),
        "CleanPing did not change any of your files. To change these settings later, run".into(),
        "cleanping setup again.".into(),
    ]);
    lines
}

fn init_line(profile: &str, shell: &str) -> Vec<String> {
    vec![
        format!("      # in {profile}"),
        format!("      eval \"$(cleanping init {shell})\""),
        String::new(),
    ]
}

#[cfg(test)]
#[path = "usage_tests.rs"]
mod tests;
