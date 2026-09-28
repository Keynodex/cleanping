//! Cleaning of provider replies before they reach a terminal, a shell line or the history.

/// Strip everything that could act on a terminal instead of being shown: control
/// characters (except newline and tab), bidirectional overrides, and self-overwriting
/// carriage returns. Line and paragraph separators become plain newlines.
pub fn clean_reply(text: &str) -> String {
    let unified = text.replace("\r\n", "\n").replace('\r', "\n");
    let cleaned: String = unified
        .chars()
        .filter_map(|c| match c {
            '\n' | '\t' => Some(c),
            '\u{2028}' | '\u{2029}' => Some('\n'),
            // C0, DEL and C1 controls, including ESC and the CSI/OSC introducers.
            c if c.is_control() => None,
            // Bidirectional embeddings, overrides and isolates can reorder what you read.
            '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}' => None,
            c if is_invisible(c) => None,
            c => Some(c),
        })
        .collect();
    cleaned.trim().to_string()
}

/// Zero-width and filler characters add nothing a person can read, but can hide what is really
/// there. The joiners and directional marks that real scripts and emoji need are kept.
fn is_invisible(c: char) -> bool {
    matches!(
        c,
        '\u{ad}'
            | '\u{61c}'
            | '\u{180e}'
            | '\u{200b}'
            | '\u{2060}'..='\u{2064}'
            | '\u{206a}'..='\u{206f}'
            | '\u{3164}'
            | '\u{feff}'
            | '\u{ffa0}'
            | '\u{e0000}'..='\u{e007f}'
    )
}

/// Names and models are shown on one line in lists and messages: no newlines, tabs or anything
/// that `clean_reply` would remove.
pub fn is_plain_line(text: &str) -> bool {
    !text.contains(['\n', '\t', '\r']) && clean_reply(text) == text
}

#[cfg(test)]
#[path = "sanitize_tests.rs"]
mod tests;
