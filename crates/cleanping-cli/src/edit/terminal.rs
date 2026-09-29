//! The real screen: `/dev/tty` in raw mode on the alternate screen. A host app runs us with
//! stdout not on the terminal, so nothing here touches stdin or stdout. Dropping it always
//! puts the terminal back.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::time::Duration;

use cleanping_core::domain::errors::{CleanpingError, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::style::{Attribute, Color, Print, SetAttribute, SetForegroundColor};
use crossterm::terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, queue};

use super::view::{Key, Line, Style};

/// What happened while we waited.
pub enum Input {
    Key(Key),
    Resized,
    Nothing,
}

pub struct Screen {
    tty: File,
}

fn unusable() -> CleanpingError {
    CleanpingError::Storage("Could not use the terminal.".into())
}

impl Screen {
    pub fn open() -> Result<Self> {
        let tty = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")
            .map_err(|_| {
                CleanpingError::Storage(
                    "There is no terminal to show the edit on. Run it from a terminal, \
                     or add --yes to edit the file without one."
                        .into(),
                )
            })?;
        terminal::enable_raw_mode().map_err(|_| unusable())?;
        let mut screen = Self { tty };
        queue!(screen.tty, EnterAlternateScreen, cursor::Hide).map_err(|_| unusable())?;
        screen.tty.flush().map_err(|_| unusable())?;
        Ok(screen)
    }

    /// Columns and rows; a terminal that reports nothing is treated as 80 by 24.
    pub fn size(&self) -> (usize, usize) {
        match terminal::size() {
            Ok((columns, rows)) if columns > 0 && rows > 0 => (columns.into(), rows.into()),
            _ => (80, 24),
        }
    }

    pub fn paint(&mut self, lines: &[Line]) -> Result<()> {
        for (row, line) in lines.iter().enumerate() {
            let row = u16::try_from(row).unwrap_or(u16::MAX);
            queue!(self.tty, cursor::MoveTo(0, row)).map_err(|_| unusable())?;
            for span in line {
                style(&mut self.tty, span.style)?;
                queue!(self.tty, Print(&span.text), SetAttribute(Attribute::Reset))
                    .map_err(|_| unusable())?;
            }
            queue!(self.tty, terminal::Clear(ClearType::UntilNewLine)).map_err(|_| unusable())?;
        }
        self.tty.flush().map_err(|_| unusable())
    }

    pub fn next_input(&mut self, wait: Duration) -> Result<Input> {
        if !event::poll(wait).map_err(|_| unusable())? {
            return Ok(Input::Nothing);
        }
        Ok(match event::read().map_err(|_| unusable())? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                key_of(key).map_or(Input::Nothing, Input::Key)
            }
            Event::Resize(..) => Input::Resized,
            _ => Input::Nothing,
        })
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        let _ = queue!(self.tty, cursor::Show, LeaveAlternateScreen);
        let _ = self.tty.flush();
        let _ = terminal::disable_raw_mode();
    }
}

fn style(tty: &mut File, style: Style) -> Result<()> {
    match style {
        Style::Plain => Ok(()),
        Style::Title => queue!(tty, SetAttribute(Attribute::Bold)),
        Style::Highlight => queue!(tty, SetAttribute(Attribute::Reverse)),
        Style::Dim => queue!(tty, SetAttribute(Attribute::Dim)),
        Style::Good => queue!(
            tty,
            SetForegroundColor(Color::Green),
            SetAttribute(Attribute::Bold)
        ),
        Style::Bad => queue!(tty, SetForegroundColor(Color::Red)),
    }
    .map_err(|_| unusable())
}

/// Ctrl+C cancels like Escape. Other Ctrl or Alt combinations do nothing, so a stray
/// Alt+N can never count as N.
fn key_of(key: KeyEvent) -> Option<Key> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return matches!(key.code, KeyCode::Char('c' | 'C')).then_some(Key::Escape);
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        return None;
    }
    match key.code {
        KeyCode::Enter => Some(Key::Enter),
        KeyCode::Esc => Some(Key::Escape),
        KeyCode::Char(c) => Some(Key::Char(c)),
        KeyCode::Up => Some(Key::Up),
        KeyCode::Down => Some(Key::Down),
        KeyCode::PageUp => Some(Key::PageUp),
        KeyCode::PageDown => Some(Key::PageDown),
        _ => None,
    }
}
