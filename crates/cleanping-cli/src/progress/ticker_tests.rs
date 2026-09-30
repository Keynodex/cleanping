use std::sync::{Arc, Mutex};

use super::*;

/// Everything the ticker wrote, shared with the test.
#[derive(Clone, Default)]
struct Captured(Arc<Mutex<Vec<u8>>>);

impl Write for Captured {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Captured {
    fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }

    fn wait_for(&self, needle: &str) {
        let started = Instant::now();
        while !self.text().contains(needle) {
            assert!(
                started.elapsed() < Duration::from_secs(10),
                "{:?}",
                self.text()
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

/// What one terminal line shows after `raw`: `\r` goes back to the start, text overwrites.
fn shown(raw: &str) -> String {
    let mut line: Vec<char> = Vec::new();
    let mut column = 0;
    for c in raw.chars() {
        if c == '\r' {
            column = 0;
            continue;
        }
        if column < line.len() {
            line[column] = c;
        } else {
            line.push(c);
        }
        column += 1;
    }
    line.into_iter().collect::<String>().trim_end().to_string()
}

const PLAIN: Look = Look {
    unicode: true,
    color: false,
};

fn pace(show_after: Duration) -> Pace {
    Pace {
        show_after,
        every: Duration::from_millis(10),
        width: || 80,
    }
}

#[test]
fn a_quick_rewrite_shows_nothing_and_does_not_wait_for_the_delay() {
    let out = Captured::default();
    let started = Instant::now();
    let ticker = Ticker::start(40, PLAIN, pace(Duration::from_secs(30)), out.clone());
    drop(ticker);
    assert!(started.elapsed() < Duration::from_secs(5));
    assert_eq!(out.text(), "");
}

#[test]
fn the_line_is_drawn_in_place_and_erased_when_dropped() {
    let out = Captured::default();
    let ticker = Ticker::start(40, PLAIN, pace(Duration::ZERO), out.clone());
    out.wait_for("about ");
    let drawn = out.text();
    assert!(
        drawn.starts_with("\rFixing your text\u{2026} "),
        "{drawn:?}"
    );
    assert!(!drawn.contains('\n'), "{drawn:?}");
    drop(ticker);
    let raw = out.text();
    assert!(raw.ends_with('\r'), "{raw:?}");
    assert_eq!(shown(&raw), "", "{raw:?}");
}

#[test]
fn the_line_is_erased_when_the_caller_panics() {
    let out = Captured::default();
    let seen = out.clone();
    let result = std::panic::catch_unwind(move || {
        let _ticker = Ticker::start(40, PLAIN, pace(Duration::ZERO), seen.clone());
        seen.wait_for("about ");
        panic!("the rewrite blew up");
    });
    assert!(result.is_err());
    let raw = out.text();
    assert!(raw.contains("about "), "positive control: {raw:?}");
    assert_eq!(shown(&raw), "", "{raw:?}");
}

#[test]
fn nothing_is_written_after_the_ticker_is_gone() {
    let out = Captured::default();
    let ticker = Ticker::start(40, PLAIN, pace(Duration::ZERO), out.clone());
    out.wait_for("about ");
    drop(ticker);
    let at_drop = out.text();
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(out.text(), at_drop);
}
