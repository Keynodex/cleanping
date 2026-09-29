//! Turns the state of the edit screen into lines: a header, the text, and a footer of keys.

use super::view::{Line, Phase, Review, Span, Style, View};
use super::wrap::{cells, clip, wrap, Cell};

fn span(text: impl Into<String>, style: Style) -> Span {
    Span {
        text: text.into(),
        style,
    }
}

fn title() -> Span {
    span("CleanPing", Style::Title)
}

fn words(text: &str) -> usize {
    text.split_whitespace().count()
}

fn summary(review: &Review) -> String {
    let Some(diff) = &review.diff else {
        return " \u{b7} too long to highlight".into();
    };
    let changed: usize = diff
        .pieces
        .iter()
        .filter(|p| p.changed)
        .map(|p| words(&p.text))
        .sum();
    let noun = if changed == 1 { "word" } else { "words" };
    match (changed, diff.removed_words) {
        (0, 0) => " \u{b7} no changes".into(),
        (n, 0) => format!(" \u{b7} {n} {noun} changed"),
        (n, removed) => format!(" \u{b7} {n} {noun} changed, {removed} removed"),
    }
}

fn header(view: &View) -> Line {
    match &view.phase {
        Phase::Waiting => vec![
            title(),
            span(
                format!("  editing with {}\u{2026}", view.provider),
                Style::Dim,
            ),
        ],
        Phase::Review(r) if r.showing_original => {
            vec![title(), span("  Original text (not edited)", Style::Dim)]
        }
        Phase::Review(r) => vec![
            title(),
            span("  Text edit complete", Style::Good),
            span(summary(r), Style::Dim),
        ],
        Phase::Failed(_) => vec![title(), span("  Could not edit", Style::Bad)],
        Phase::Refused(_) => vec![title(), span("  Not sent", Style::Bad)],
    }
}

fn footer(view: &View, scrollable: bool) -> Line {
    let scroll = if scrollable {
        " \u{b7} \u{2191}\u{2193} scroll"
    } else {
        ""
    };
    let text = match &view.phase {
        Phase::Waiting => format!("Esc cancel{scroll}"),
        Phase::Review(r) if r.showing_original => {
            format!("Enter accept the edit \u{b7} N keep original \u{b7} O view edited{scroll}")
        }
        Phase::Review(_) => {
            format!("Enter accept \u{b7} N keep original \u{b7} O view original{scroll}")
        }
        Phase::Failed(_) => "Press any key to keep your text as it is".into(),
        Phase::Refused(_) => "S send anyway \u{b7} any other key keeps your text".into(),
    };
    vec![span(text, Style::Dim)]
}

fn body(view: &View) -> Vec<Cell> {
    match &view.phase {
        Phase::Waiting => cells(&view.original, Style::Highlight),
        Phase::Review(r) if r.showing_original => cells(&view.original, Style::Plain),
        Phase::Review(r) => match &r.diff {
            Some(diff) => diff
                .pieces
                .iter()
                .flat_map(|p| {
                    cells(
                        &p.text,
                        if p.changed {
                            Style::Highlight
                        } else {
                            Style::Plain
                        },
                    )
                })
                .collect(),
            None => cells(&r.edited, Style::Plain),
        },
        Phase::Failed(message) => [
            cells(message, Style::Bad),
            cells("\n\nYour text is unchanged:\n", Style::Dim),
            cells(&view.original, Style::Plain),
        ]
        .concat(),
        Phase::Refused(kind) => [
            cells(
                &format!("This looks like it contains {kind}.\n"),
                Style::Plain,
            ),
            cells(
                &format!("It was not sent to {}.\n\n", view.provider),
                Style::Plain,
            ),
            cells(
                "Sending it anyway shares it with that provider.",
                Style::Dim,
            ),
        ]
        .concat(),
    }
}

/// Exactly `height` lines of at most `width` characters. Keeps the scroll position inside the
/// text.
pub fn render(view: &mut View, width: usize, height: usize) -> Vec<Line> {
    let height = height.max(4);
    let rows = wrap(&body(view), width);
    let room = height - 3;
    view.scroll = view.scroll.min(rows.len().saturating_sub(room));
    let mut lines = vec![header(view), Vec::new()];
    lines.extend(rows.iter().skip(view.scroll).take(room).cloned());
    lines.resize(height - 1, Vec::new());
    lines.push(footer(view, rows.len() > room));
    lines.into_iter().map(|line| clip(line, width)).collect()
}
