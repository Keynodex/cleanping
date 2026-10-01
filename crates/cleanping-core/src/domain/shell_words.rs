//! Splitting a command into words the way a POSIX shell quotes them, and noticing a quote that
//! is still open at the end. Small on purpose: no expansions, no heredocs, no `$'...'`.

/// One word of a command, quotes and backslashes kept as typed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Word {
    /// The word as typed.
    pub text: String,
    /// True when the word starts with a quote or a backtick.
    pub quoted: bool,
}

/// A command split into words, and the quote still open at its end, if any.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scan {
    /// The words, in order.
    pub words: Vec<Word>,
    /// `'`, `"` or `` ` `` when that quote is never closed.
    pub open: Option<char>,
}

/// Reads a command piece by piece, so lines can be added while the quote state carries over.
#[derive(Debug, Default)]
pub struct Scanner {
    scan: Scan,
    current: String,
    quoted: bool,
    comment: bool,
    escaped: bool,
    previous: Option<char>,
}

impl Scanner {
    /// Read more of the command.
    pub fn feed(&mut self, text: &str) {
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            self.step(c, chars.peek().copied());
            self.previous = Some(c);
        }
    }

    /// The quote still open so far, if any.
    pub fn open(&self) -> Option<char> {
        self.scan.open
    }

    /// True when the text so far ends in a backslash that escapes the next line.
    pub fn continues(&self) -> bool {
        self.escaped || self.scan.open.is_some()
    }

    /// The words and the open quote.
    pub fn finish(mut self) -> Scan {
        self.end_word();
        self.scan
    }

    fn step(&mut self, c: char, next: Option<char>) {
        if self.comment {
            self.comment = c != '\n';
            return;
        }
        if self.escaped {
            self.escaped = false;
            if c == '\n' {
                self.current.pop(); // a backslash-newline joins the lines and is not kept
            } else {
                self.current.push(c);
            }
            return;
        }
        match self.scan.open {
            Some('\'') => self.inside(c, '\''),
            Some(_) if c == '\\' => {
                self.current.push(c);
                self.escaped = true;
            }
            Some(quote) => self.inside(c, quote),
            None => self.outside(c, next),
        }
    }

    fn inside(&mut self, c: char, quote: char) {
        self.current.push(c);
        if c == quote {
            self.scan.open = None;
        }
    }

    fn outside(&mut self, c: char, next: Option<char>) {
        match c {
            '\\' => {
                self.current.push(c);
                self.escaped = true;
            }
            '#' if self.current.is_empty() => self.comment = true,
            '\'' if is_apostrophe(self.previous, next) => self.current.push(c),
            '\'' | '"' | '`' => {
                self.quoted |= self.current.is_empty();
                self.current.push(c);
                self.scan.open = Some(c);
            }
            c if c.is_whitespace() => self.end_word(),
            c => self.current.push(c),
        }
    }

    fn end_word(&mut self) {
        if !self.current.is_empty() {
            self.scan.words.push(Word {
                text: std::mem::take(&mut self.current),
                quoted: self.quoted,
            });
        }
        self.quoted = false;
    }
}

/// `don't`, `it's`: a single quote between two letters is read as an apostrophe. A shell would
/// see a quote there; this is the trade that keeps prose from looking like a broken command.
fn is_apostrophe(previous: Option<char>, next: Option<char>) -> bool {
    previous.is_some_and(char::is_alphabetic) && next.is_some_and(char::is_alphabetic)
}

/// Split a whole command at once.
pub fn scan(command: &str) -> Scan {
    let mut scanner = Scanner::default();
    scanner.feed(command);
    scanner.finish()
}

#[cfg(test)]
#[path = "shell_words_tests.rs"]
mod tests;
