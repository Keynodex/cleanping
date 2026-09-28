//! Best-effort clipboard copy through a system tool (fixed argv, no shell).

use std::io::Write;
use std::process::{Command, Stdio};

const TOOLS: &[(&str, &[&str])] = &[
    ("wl-copy", &[]),
    ("xclip", &["-selection", "clipboard"]),
    ("xsel", &["--clipboard", "--input"]),
    ("pbcopy", &[]),
];

/// True when some tool accepted the text.
pub fn copy(text: &str) -> bool {
    TOOLS.iter().any(|(program, args)| {
        let Ok(mut child) = Command::new(program)
            .args(*args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        else {
            return false;
        };
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        child.wait().is_ok_and(|status| status.success())
    })
}

pub const TOOL_NAMES: &str = "wl-copy, xclip, xsel, pbcopy";
