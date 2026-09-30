//! Whether this terminal gets the progress line, and in which characters and colors. Pure: the
//! environment comes in as a lookup, so every case is tested without touching the real one.

use super::bar::Look;

/// `CLEANPING_PROGRESS=off` (or `0`) turns the line off.
pub const SWITCH: &str = "CLEANPING_PROGRESS";

/// The look of the line, or `None` when it must not be shown: stderr is not a terminal (a
/// pipe, a file, a script, the shell key) or the user turned it off.
pub fn look_for(var: impl Fn(&str) -> Option<String>, stderr_is_terminal: bool) -> Option<Look> {
    let set = |name: &str| var(name).filter(|value| !value.is_empty());
    let off = set(SWITCH).is_some_and(|value| {
        let value = value.trim();
        value == "0" || value.eq_ignore_ascii_case("off")
    });
    if !stderr_is_terminal || off {
        return None;
    }
    let dumb = set("TERM").is_some_and(|term| term == "dumb");
    // The first of these that is set decides the character set, as the C library does.
    let locale = ["LC_ALL", "LC_CTYPE", "LANG"].into_iter().find_map(set);
    let utf8 = locale.is_some_and(|name| {
        let name = name.to_ascii_lowercase();
        name.contains("utf-8") || name.contains("utf8")
    });
    Some(Look {
        unicode: utf8 && !dumb,
        color: !dumb && set("NO_COLOR").is_none(),
    })
}

#[cfg(test)]
#[path = "look_tests.rs"]
mod tests;
