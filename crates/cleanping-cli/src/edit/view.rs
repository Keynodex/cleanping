//! What the edit screen shows and how keys act on it. Pure: no terminal, no network, so every
//! state can be tested by looking at plain lines.

use std::time::Duration;

use cleanping_core::domain::diff::Diff;

/// How a piece of text is drawn; the terminal layer maps these to colors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Plain,
    Title,
    Highlight,
    Dim,
    Good,
    Bad,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub style: Style,
}

pub type Line = Vec<Span>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Enter,
    Escape,
    Char(char),
    Up,
    Down,
    PageUp,
    PageDown,
}

/// What the caller should do after a key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Continue,
    /// Put this text in the file.
    Accept(String),
    KeepOriginal,
    /// The user chose to send text that looked like it held a secret.
    SendAnyway,
}

#[derive(Clone, Debug)]
pub struct Review {
    pub edited: String,
    pub diff: Option<Diff>,
    pub showing_original: bool,
    /// How the edit changed a command in the text, in words, if it did.
    pub command_warning: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Phase {
    /// The text looks like it holds this kind of secret; nothing was sent.
    Refused(&'static str),
    Waiting,
    Review(Review),
    Failed(String),
}

#[derive(Clone, Debug)]
pub struct View {
    pub original: String,
    /// For example "DeepSeek (api.deepseek.com)".
    pub provider: String,
    pub phase: Phase,
    pub scroll: usize,
    /// How long the reply has been awaited; the estimate shows once it is long enough.
    pub waited: Duration,
}

impl View {
    pub fn new(original: String, provider: String, phase: Phase) -> Self {
        Self {
            original,
            provider,
            phase,
            scroll: 0,
            waited: Duration::ZERO,
        }
    }

    /// Exactly `height` lines, none wider than `width` characters.
    pub fn render(&mut self, width: usize, height: usize) -> Vec<Line> {
        super::layout::render(self, width, height)
    }

    /// `page` is how many lines PageUp and PageDown move.
    pub fn on_key(&mut self, key: Key, page: usize) -> Action {
        if matches!(self.phase, Phase::Waiting | Phase::Review(_)) {
            match key {
                Key::Up => self.scroll = self.scroll.saturating_sub(1),
                Key::Down => self.scroll += 1,
                Key::PageUp => self.scroll = self.scroll.saturating_sub(page),
                Key::PageDown => self.scroll += page,
                _ => {}
            }
        }
        let leave = matches!(key, Key::Escape | Key::Char('n' | 'N'));
        match &mut self.phase {
            Phase::Refused(_) if matches!(key, Key::Char('s' | 'S')) => Action::SendAnyway,
            Phase::Refused(_) | Phase::Failed(_) => Action::KeepOriginal,
            Phase::Waiting if leave => Action::KeepOriginal,
            Phase::Review(_) if leave => Action::KeepOriginal,
            Phase::Review(review) => match key {
                Key::Enter => Action::Accept(review.edited.clone()),
                Key::Char('o' | 'O') => {
                    review.showing_original = !review.showing_original;
                    Action::Continue
                }
                _ => Action::Continue,
            },
            Phase::Waiting => Action::Continue,
        }
    }
}

#[cfg(test)]
#[path = "view_tests.rs"]
mod tests;
