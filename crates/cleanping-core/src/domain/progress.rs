//! An honest guess at how far a rewrite has got. The request is one call that answers all at
//! once, so the real progress is unknown until the reply arrives; this only estimates it from the
//! size of the text and the time spent waiting.

use std::time::Duration;

/// How far a rewrite probably is, as a whole percent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Progress {
    /// From 0 up to [`HELD_AT`]. Never 100: only the reply itself can say the rewrite is done.
    pub percent: u8,
}

/// The estimate stops here and waits for the reply, however long it takes.
pub const HELD_AT: u8 = 95;

/// The timing guesses behind the estimate, in one place so they can be tuned. They are still a
/// guess: retune them from real timings. Most of the wait is the model writing its reply, which
/// is about as long as the text.
///
/// Measured 2026-10-01 with a fast provider (DeepSeek flash, thinking off): a 5,934-character
/// text took 4.8 to 5.1 s and a short one 0.5 to 0.7 s, about 0.6 s plus 0.7 s a kilobyte. These
/// add some room to that, so the 5,934-byte text is expected in about 6.4 s. A slower provider
/// (a thinking model, a local one) waits longer at the hold.
pub mod timing {
    use std::time::Duration;

    /// Connecting and the model's first word, whatever the size of the text. With this, a
    /// short text is just over half way when the line first shows at 0.5 s.
    pub const BASE: Duration = Duration::from_secs(1);
    /// About 1,100 bytes a second, a fast hosted model.
    pub const PER_KILOBYTE: Duration = Duration::from_millis(900);
    /// No request waits longer than the HTTP timeout (`http_rewriter::DEFAULT_TIMEOUT`).
    pub const LONGEST: Duration = Duration::from_secs(180);
}

/// How long a rewrite of `input_bytes` is expected to take: [`timing::BASE`] plus
/// [`timing::PER_KILOBYTE`] for each 1,000 bytes, at most [`timing::LONGEST`].
pub fn expected_duration(input_bytes: usize) -> Duration {
    let per_byte = timing::PER_KILOBYTE.as_secs_f64() / 1000.0;
    let grow = per_byte * input_bytes as f64;
    let total = timing::BASE.as_secs_f64() + grow;
    Duration::from_secs_f64(total.min(timing::LONGEST.as_secs_f64()))
}

/// The estimated progress after waiting `elapsed` for a rewrite of `input_bytes`.
///
/// Starts at 0, rises quickly and then more slowly (about 83 % at the expected time), and
/// reaches [`HELD_AT`] at twice the expected time, where it stays. It never goes down as
/// `elapsed` grows, and a bigger text is never further along after the same wait.
pub fn estimate_progress(input_bytes: usize, elapsed: Duration) -> Progress {
    let expected = expected_duration(input_bytes).as_secs_f64();
    // Share of the way to twice the expected time, where the estimate stops.
    let share = (elapsed.as_secs_f64() / (2.0 * expected)).min(1.0);
    // Ease out: fast at first, slower near the end, smooth where it meets the hold.
    let eased = 1.0 - (1.0 - share).powi(3);
    let percent = (f64::from(HELD_AT) * eased).floor();
    Progress {
        percent: (percent as u8).min(HELD_AT),
    }
}

#[cfg(test)]
#[path = "progress_tests.rs"]
mod tests;
