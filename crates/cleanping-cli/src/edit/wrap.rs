//! Cutting styled text into screen rows. Widths are counted in characters, so wide characters
//! (for example Chinese) can push a row a little past the edge; the terminal then wraps it.

use super::view::{Line, Span, Style};

pub type Cell = (char, Style);

/// Text made safe to print: tabs become spaces and other control characters become a dot, so
/// text from a file can never send commands to the terminal.
pub fn printable(text: &str) -> String {
    text.chars()
        .flat_map(|c| match c {
            '\n' => vec!['\n'],
            '\t' => vec![' '; 4],
            c if c.is_control() || c == '\r' => vec!['\u{b7}'],
            c => vec![c],
        })
        .collect()
}

/// Every character of `text` with the same style.
pub fn cells(text: &str, style: Style) -> Vec<Cell> {
    printable(text).chars().map(|c| (c, style)).collect()
}

/// Rows of at most `width` characters. A row ends at a line break, or after the last space that
/// fits, or, for a very long word, at the width.
pub fn wrap(input: &[Cell], width: usize) -> Vec<Line> {
    let width = width.max(1);
    input
        .split(|(c, _)| *c == '\n')
        .flat_map(|paragraph| wrap_paragraph(paragraph, width))
        .collect()
}

fn wrap_paragraph(paragraph: &[Cell], width: usize) -> Vec<Line> {
    if paragraph.is_empty() {
        return vec![Vec::new()];
    }
    let mut rows = Vec::new();
    let mut start = 0;
    while paragraph.len() - start > width {
        let window = &paragraph[start..start + width];
        let cut = window
            .iter()
            .rposition(|(c, _)| *c == ' ')
            .map_or(width, |space| space + 1);
        rows.push(compact(&paragraph[start..start + cut]));
        start += cut;
    }
    rows.push(compact(&paragraph[start..]));
    rows
}

/// Neighbouring characters with the same style become one span.
fn compact(row: &[Cell]) -> Line {
    let mut line: Line = Vec::new();
    for (c, style) in row {
        match line.last_mut() {
            Some(last) if last.style == *style => last.text.push(*c),
            _ => line.push(Span {
                text: c.to_string(),
                style: *style,
            }),
        }
    }
    line
}

/// Shorten a line to `width` characters.
pub fn clip(line: Line, width: usize) -> Line {
    let mut left = width;
    let mut out = Vec::new();
    for span in line {
        if left == 0 {
            break;
        }
        let text: String = span.text.chars().take(left).collect();
        left -= text.chars().count();
        out.push(Span {
            text,
            style: span.style,
        });
    }
    out
}
