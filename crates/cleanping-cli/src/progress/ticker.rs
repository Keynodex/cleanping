//! Draws the progress line from a background thread until it is dropped. Dropping it stops the
//! thread, erases the line and waits for both, so whatever the caller prints next (the result,
//! an error, a refusal) starts on a clean line. It is dropped on every path, a panic included.

use std::io::Write;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use cleanping_core::domain::progress::estimate_progress;

use super::bar::{render_bar, Look};

/// When the line first appears, how often it changes, and how wide the terminal is.
#[derive(Clone, Copy)]
pub struct Pace {
    pub show_after: Duration,
    pub every: Duration,
    pub width: fn() -> usize,
}

/// Keeps the line on screen while it lives.
pub struct Ticker {
    stop: Option<Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl Ticker {
    /// Draw the estimate for a text of `input_bytes` on `out` with `look`.
    pub fn start<W: Write + Send + 'static>(
        input_bytes: usize,
        look: Look,
        pace: Pace,
        out: W,
    ) -> Self {
        let started = Instant::now();
        let (stop, stopped) = mpsc::channel::<()>();
        let thread = std::thread::spawn(move || {
            let mut out = out;
            if stopped.recv_timeout(pace.show_after) != Err(RecvTimeoutError::Timeout) {
                return;
            }
            // The widest line so far: the spaces that erase it when the rewrite ends.
            let mut widest: usize = 0;
            loop {
                let (line, used) = frame(input_bytes, started.elapsed(), (pace.width)(), look);
                let pad = " ".repeat(widest.saturating_sub(used));
                widest = widest.max(used);
                draw(&mut out, &format!("\r{line}{pad}"));
                if stopped.recv_timeout(pace.every) != Err(RecvTimeoutError::Timeout) {
                    break;
                }
            }
            // Spaces rather than an erase sequence, so a dumb terminal is cleared too.
            draw(&mut out, &format!("\r{}\r", " ".repeat(widest)));
        });
        Self {
            stop: Some(stop),
            thread: Some(thread),
        }
    }
}

fn draw<W: Write>(out: &mut W, text: &str) {
    let _ = out.write_all(text.as_bytes()).and_then(|()| out.flush());
}

/// The line, and how many columns it takes on screen (the color codes take none).
fn frame(input_bytes: usize, elapsed: Duration, width: usize, look: Look) -> (String, usize) {
    let progress = estimate_progress(input_bytes, elapsed);
    let plain = Look {
        color: false,
        ..look
    };
    let used = render_bar(progress, elapsed, width, plain).chars().count();
    (render_bar(progress, elapsed, width, look), used)
}

impl Drop for Ticker {
    fn drop(&mut self) {
        drop(self.stop.take());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
#[path = "ticker_tests.rs"]
mod tests;
