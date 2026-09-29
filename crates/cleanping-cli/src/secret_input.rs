//! Reading an API key in a terminal. Only a star per character is shown, Ctrl-C and Esc cancel,
//! and the terminal always gets its typing back, however reading ends. (A library prompt that
//! turns echo off cannot promise that: Ctrl-C kills the program before it can turn echo on.)

use std::fs::OpenOptions;
use std::io::Write;

use cleanping_core::domain::errors::{CleanpingError, Result};
use cleanping_core::domain::validation::MAX_API_KEY_BYTES;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal;

fn invalid(message: &str) -> CleanpingError {
    CleanpingError::Validation(message.to_string())
}

/// Turns raw mode off again when dropped.
struct RawMode;

impl RawMode {
    fn enter() -> Result<Self> {
        terminal::enable_raw_mode()
            .map_err(|_| invalid("Could not read the key from the terminal."))?;
        Ok(Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

/// Ask for a key on the terminal (`/dev/tty`, so it works even when stdout is redirected).
/// Enter finishes; Backspace and Ctrl-U edit; Ctrl-C and Esc cancel with an error.
pub fn read_secret(prompt: &str) -> Result<String> {
    let mut tty = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .map_err(|_| invalid("Could not read the key: there is no terminal."))?;
    let _ = write!(tty, "{prompt}");
    let _ = tty.flush();
    let raw = RawMode::enter()?;
    let mut secret = String::new();
    let outcome = loop {
        let Ok(Event::Key(key)) = event::read() else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        match key.code {
            // In raw mode Enter is CR, and a line feed arrives as Ctrl-J.
            KeyCode::Enter => break Ok(()),
            KeyCode::Char('j') if ctrl => break Ok(()),
            KeyCode::Esc => break Err(()),
            KeyCode::Char('c') if ctrl => break Err(()),
            KeyCode::Backspace => erase(&mut tty, &mut secret, 1),
            KeyCode::Char('h') if ctrl => erase(&mut tty, &mut secret, 1),
            KeyCode::Char('u') if ctrl => {
                let all = secret.chars().count();
                erase(&mut tty, &mut secret, all);
            }
            // One byte over the limit is kept, so the too-long message can still appear.
            KeyCode::Char(c) if !ctrl && !alt && secret.len() <= MAX_API_KEY_BYTES => {
                secret.push(c);
                let _ = write!(tty, "*");
                let _ = tty.flush();
            }
            _ => {}
        }
    };
    drop(raw);
    let _ = writeln!(tty);
    match outcome {
        Ok(()) => Ok(secret),
        Err(()) => Err(invalid("Cancelled: no key was entered.")),
    }
}

/// Remove the last `count` characters and their stars.
fn erase(tty: &mut impl Write, secret: &mut String, count: usize) {
    for _ in 0..count {
        if secret.pop().is_some() {
            let _ = write!(tty, "\x08 \x08");
        }
    }
    let _ = tty.flush();
}
