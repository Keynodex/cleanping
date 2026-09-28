//! `cleanping history ...`: look at, and delete, the local rewrite history.

use cleanping_core::domain::errors::{CleanpingError, Result};
use cleanping_core::domain::models::Run;
use cleanping_core::domain::sanitize::clean_reply;
use cleanping_core::infrastructure::clock::days_ago;

use crate::args::HistoryAction;
use crate::output;
use crate::services::Services;

const PREVIEW_CHARS: usize = 60;

/// Anything read from the database is printed on a terminal: whitespace collapsed, control
/// and hidden characters removed. The rows may come from an older program or a hand edit.
fn one_line(text: &str) -> String {
    clean_reply(&text.split_whitespace().collect::<Vec<_>>().join(" "))
}

/// One line of the user's text, shortened.
fn preview(text: &str) -> String {
    let single_line = one_line(text);
    match single_line.char_indices().nth(PREVIEW_CHARS) {
        Some((end, _)) => format!("{}\u{2026}", &single_line[..end]),
        None => single_line,
    }
}

fn when(run: &Run) -> String {
    let stamp = one_line(run.created_at.as_deref().unwrap_or(""));
    match stamp.get(..16) {
        Some(minute) => format!("{} UTC", minute.replace('T', " ")),
        None => stamp,
    }
}

fn deleted(count: usize) -> String {
    let noun = if count == 1 { "run" } else { "runs" };
    format!("Deleted {count} {noun}.")
}

pub fn run(services: &Services, action: &HistoryAction) -> Result<()> {
    match action {
        HistoryAction::List { limit } => {
            for run in services.history.recent(*limit)? {
                output::line(&format!(
                    "{}  {:<5} {}  {}",
                    when(&run),
                    run.status.as_str(),
                    one_line(&run.model),
                    preview(&run.input_text)
                ))?;
            }
            Ok(())
        }
        HistoryAction::Clear { yes } => {
            if !yes {
                return Err(CleanpingError::Validation(
                    "This deletes every saved rewrite. Run again with --yes to confirm.".into(),
                ));
            }
            output::line(&deleted(services.history.clear()?))
        }
        HistoryAction::Purge { older_than } => {
            let removed = services.history.purge_before(&days_ago(*older_than))?;
            output::line(&deleted(removed))
        }
    }
}
