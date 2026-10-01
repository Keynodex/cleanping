use super::*;
use cleanping_core::domain::diff::highlight_changes;

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
