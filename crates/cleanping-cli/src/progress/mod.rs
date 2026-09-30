//! The progress line shown on stderr while a rewrite is on its way. The request answers all at
//! once, so the percent is an estimate (`cleanping_core::domain::progress`); the seconds are
//! real. Never on stdout, and only when stderr is a terminal.

pub mod bar;
pub mod look;
pub mod ticker;

use std::io::IsTerminal;
use std::time::Duration;

use ticker::{Pace, Ticker};

const EVERY: Duration = Duration::from_millis(100);

/// Columns of the terminal; one that reports nothing counts as 80.
fn width() -> usize {
    match crossterm::terminal::size() {
        Ok((columns, _)) if columns > 0 => columns.into(),
        _ => 80,
    }
}

/// Show the estimate for a text of `input_bytes` on stderr until the returned value is
/// dropped, which erases it. `None` (nothing shown) unless stderr is a terminal and
/// `CLEANPING_PROGRESS` is not `off`. Drop it before printing anything.
pub fn while_waiting(input_bytes: usize) -> Option<Ticker> {
    let stderr = std::io::stderr();
    let look = look::look_for(|name| std::env::var(name).ok(), stderr.is_terminal())?;
    let pace = Pace {
        show_after: bar::SHOW_AFTER,
        every: EVERY,
        width,
    };
    Some(Ticker::start(input_bytes, look, pace, stderr))
}
