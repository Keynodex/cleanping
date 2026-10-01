//! The words and blocks of the progress line. Pure: progress and time in, text out.

use std::time::Duration;

use cleanping_core::domain::progress::Progress;

/// Quick rewrites show nothing: the line appears only after this long.
pub const SHOW_AFTER: Duration = Duration::from_millis(1500);

const GREEN: &str = "\u{1b}[32m";
const RESET: &str = "\u{1b}[0m";
/// The longest tail, `about 95% 180s`, so the bar keeps its length as the seconds grow.
const TAIL_ROOM: usize = 14;
const MOST_BLOCKS: usize = 20;
/// With fewer blocks than this the words go, to leave room for the bar.
const BLOCKS_WITH_LABEL: usize = 6;
const FEWEST_BLOCKS: usize = 4;

/// How the line may be drawn on this terminal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Look {
    /// `▰▱` and `…`; otherwise `[##..]` and `...`.
    pub unicode: bool,
    /// The filled blocks in green.
    pub color: bool,
}

/// The pieces of the line, for callers that style them themselves (the edit screen).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BarText {
    /// `Fixing your text… ` (left out when the terminal is narrow), and `[` in ASCII.
    pub label: String,
    /// The filled blocks; with the empty ones, nothing when there is no room for a bar.
    pub filled: String,
    pub empty: String,
    /// For example ` about 40% 12s` (`]` first in ASCII): the percent is an estimate, the
    /// seconds are real.
    pub tail: String,
}

fn label(unicode: bool) -> &'static str {
    if unicode {
        "Fixing your text\u{2026} "
    } else {
        "Fixing your text... "
    }
}

/// The pieces of a line that fits in `width` columns with the last column left free.
pub fn bar_text(progress: Progress, elapsed: Duration, width: usize, unicode: bool) -> BarText {
    let usable = width.saturating_sub(1);
    let tail = format!("about {}% {}s", progress.percent, elapsed.as_secs());
    let brackets = if unicode { 0 } else { 2 };
    let room = usable.saturating_sub(1 + TAIL_ROOM + brackets);
    let label = label(unicode);
    let (label, blocks) = match room.saturating_sub(label.chars().count()) {
        blocks if blocks >= BLOCKS_WITH_LABEL => (label, blocks),
        _ if room >= FEWEST_BLOCKS => ("", room),
        _ => {
            let tail = tail.chars().take(usable).collect();
            return BarText {
                label: String::new(),
                filled: String::new(),
                empty: String::new(),
                tail,
            };
        }
    };
    let blocks = blocks.min(MOST_BLOCKS);
    let filled = blocks * usize::from(progress.percent) / 100;
    let (full, open, close, none) = if unicode {
        ("\u{25b0}", "", "", "\u{25b1}")
    } else {
        ("#", "[", "]", ".")
    };
    BarText {
        label: format!("{label}{open}"),
        filled: full.repeat(filled),
        empty: none.repeat(blocks - filled),
        tail: format!("{close} {tail}"),
    }
}

/// The whole line, for example `Fixing your text… ▰▰▰▰▰▰▰▰▱▱▱▱▱▱▱▱▱▱▱▱ about 40% 12s`.
pub fn render_bar(progress: Progress, elapsed: Duration, width: usize, look: Look) -> String {
    let text = bar_text(progress, elapsed, width, look.unicode);
    let filled = if look.color && !text.filled.is_empty() {
        format!("{GREEN}{}{RESET}", text.filled)
    } else {
        text.filled
    };
    format!("{}{filled}{}{}", text.label, text.empty, text.tail)
}

#[cfg(test)]
#[path = "bar_tests.rs"]
mod tests;
