use super::*;
use cleanping_core::domain::diff::highlight_changes;
use cleanping_core::domain::progress::estimate_progress;

const WARNING: &str = "a quote was closed in the command starting `curl -d`";

fn review(warning: Option<&str>, showing_original: bool) -> View {
    let (original, edited) = ("curl -d 'x", "curl -d 'x'");
    View::new(
        original.into(),
        "Local".into(),
        Phase::Review(Review {
            edited: edited.into(),
            diff: highlight_changes(original, edited),
            showing_original,
            command_warning: warning.map(String::from),
        }),
    )
}

fn text(line: &Line) -> String {
    line.iter().map(|s| s.text.as_str()).collect()
}

#[test]
fn a_changed_command_is_named_on_the_line_under_the_header() {
    let lines = render(&mut review(Some(WARNING), false), 100, 10);
    assert_eq!(lines.len(), 10);
    assert_eq!(text(&lines[1]), format!("Check the command: {WARNING}"));
    assert!(
        lines[1].iter().all(|s| s.style == Style::Bad),
        "{:?}",
        lines[1]
    );
    assert!(text(&lines[0]).contains("Text edit complete"));
}

#[test]
fn the_warning_stays_while_the_original_is_shown() {
    let lines = render(&mut review(Some(WARNING), true), 100, 10);
    assert!(text(&lines[1]).starts_with("Check the command:"));
}

#[test]
fn the_warning_is_cut_to_the_screen_width() {
    let lines = render(&mut review(Some(WARNING), false), 30, 10);
    assert_eq!(text(&lines[1]).chars().count(), 30);
}

#[test]
fn without_a_changed_command_the_line_stays_blank() {
    let lines = render(&mut review(None, false), 100, 10);
    assert_eq!(text(&lines[1]), "");
}

fn waiting(seconds: u64) -> View {
    let mut view = View::new("pls fix teh login".into(), "Local".into(), Phase::Waiting);
    view.waited = std::time::Duration::from_secs(seconds);
    view
}

#[test]
fn a_slow_edit_shows_the_estimate_under_the_header() {
    let lines = render(&mut waiting(4), 80, 10);
    assert!(text(&lines[0]).contains("editing with Local"));
    let notice = text(&lines[1]);
    assert!(notice.starts_with('\u{25b0}'), "{notice}");
    assert!(notice.contains('\u{25b1}'), "{notice}");
    let percent = estimate_progress(17, std::time::Duration::from_secs(4)).percent;
    assert!(
        notice.ends_with(&format!(" about {percent}% 4s")),
        "{notice}"
    );
    let styles: Vec<Style> = lines[1].iter().map(|s| s.style).collect();
    assert_eq!(
        styles,
        [Style::Good, Style::Dim, Style::Dim],
        "{:?}",
        lines[1]
    );
}

#[test]
fn a_quick_edit_shows_no_estimate() {
    let lines = render(&mut waiting(1), 80, 10);
    assert_eq!(text(&lines[1]), "");
}

#[test]
fn the_estimate_fits_a_narrow_screen() {
    let lines = render(&mut waiting(4), 20, 10);
    let notice = text(&lines[1]);
    assert!(notice.contains("% 4s"), "{notice}");
    assert!(notice.contains('\u{25b0}'), "{notice}");
    assert!(notice.chars().count() < 20, "{notice}");
}
