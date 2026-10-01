//! The estimate on the edit screen while the reply is on its way: the same bar and words as the
//! plain command, as styled spans for the line under the header. Pure.

use std::time::Duration;

use cleanping_core::domain::progress::estimate_progress;

use super::view::{Line, Span, Style};
use crate::progress::bar::{bar_text, SHOW_AFTER};

/// Nothing for a quick edit; after [`SHOW_AFTER`], the green bar and `about 40% 12s`. The
/// header already says what is happening, so the words before the bar are left out.
pub fn line(text_bytes: usize, waited: Duration, width: usize) -> Line {
    if waited < SHOW_AFTER {
        return Vec::new();
    }
    let parts = bar_text(estimate_progress(text_bytes, waited), waited, width, true);
    vec![
        Span {
            text: parts.filled,
            style: Style::Good,
        },
        Span {
            text: parts.empty,
            style: Style::Dim,
        },
        Span {
            text: parts.tail,
            style: Style::Dim,
        },
    ]
}
