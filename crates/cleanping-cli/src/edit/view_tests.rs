use super::*;
use cleanping_core::domain::diff::highlight_changes;

const PROVIDER: &str = "DeepSeek (api.deepseek.com)";

fn plain(lines: &[Line]) -> Vec<String> {
    lines
        .iter()
        .map(|line| line.iter().map(|s| s.text.as_str()).collect::<String>())
        .collect()
}

fn highlighted(lines: &[Line]) -> String {
    lines
        .iter()
        .flatten()
        .filter(|s| s.style == Style::Highlight)
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join("|")
}

fn waiting(text: &str) -> View {
    View::new(text.into(), PROVIDER.into(), Phase::Waiting)
}

fn review(original: &str, edited: &str) -> View {
    View::new(
        original.into(),
        PROVIDER.into(),
        Phase::Review(Review {
            edited: edited.into(),
            diff: highlight_changes(original, edited),
            showing_original: false,
        }),
    )
}

#[test]
fn every_render_is_exactly_the_screen_height_and_never_too_wide() {
    let long = "word ".repeat(80);
    for mut view in [
        waiting(&long),
        review(&long, &format!("{long}more")),
        waiting(""),
    ] {
        for (width, height) in [(80, 24), (40, 10), (20, 6), (120, 50)] {
            let lines = plain(&view.render(width, height));
            assert_eq!(lines.len(), height);
            assert!(
                lines.iter().all(|l| l.chars().count() <= width),
                "{width}x{height}"
            );
        }
    }
}

#[test]
fn while_waiting_the_whole_text_is_lit_up_and_the_provider_is_named() {
    let mut view = waiting("pleae fix teh login");
    let lines = view.render(80, 12);
    let text = plain(&lines);
    assert!(
        text[0].contains("CleanPing") && text[0].contains(PROVIDER),
        "{}",
        text[0]
    );
    assert!(text[0].contains("editing"), "{}", text[0]);
    assert_eq!(highlighted(&lines), "pleae fix teh login");
    assert!(text.last().unwrap().contains("Esc"), "{:?}", text.last());
}

#[test]
fn a_finished_edit_says_so_and_lights_up_only_the_changed_words() {
    let mut view = review("pleae fix teh login", "Please fix the login.");
    let lines = view.render(80, 12);
    let text = plain(&lines);
    assert!(text[0].contains("Text edit complete"), "{}", text[0]);
    assert!(text.iter().any(|l| l.contains("Please fix the login.")));
    assert_eq!(highlighted(&lines), "Please|the login.");
    let footer = text.last().unwrap();
    assert!(
        footer.contains("Enter") && footer.contains("N") && footer.contains("original"),
        "{footer}"
    );
}

#[test]
fn the_header_counts_changes() {
    let mut view = review("a b c d", "a x y d");
    let header = plain(&view.render(80, 12))[0].clone();
    assert!(header.contains("2 words changed"), "{header}");
    assert!(header.contains("2 removed"), "{header}");
}

#[test]
fn a_text_too_large_to_compare_is_shown_without_highlights_and_says_so() {
    let mut view = review("a", "b");
    if let Phase::Review(r) = &mut view.phase {
        r.diff = None;
    }
    let lines = view.render(80, 12);
    assert_eq!(highlighted(&lines), "");
    assert!(plain(&lines)[0].contains("too long to highlight"));
}

#[test]
fn o_switches_between_the_edit_and_the_original() {
    let mut view = review("pleae fix", "Please fix");
    assert_eq!(view.on_key(Key::Char('o'), 10), Action::Continue);
    let shown = plain(&view.render(80, 12));
    assert!(shown[0].contains("Original"), "{}", shown[0]);
    assert!(shown.iter().any(|l| l.contains("pleae fix")));
    assert!(!shown.iter().any(|l| l.contains("Please fix")));
    assert!(shown.last().unwrap().contains("view edited"));
    assert_eq!(view.on_key(Key::Char('O'), 10), Action::Continue);
    assert!(plain(&view.render(80, 12))[0].contains("Text edit complete"));
}

#[test]
fn enter_accepts_and_n_or_escape_keep_the_original() {
    let mut view = review("a", "A.");
    assert_eq!(view.on_key(Key::Enter, 10), Action::Accept("A.".into()));
    for key in [Key::Char('n'), Key::Char('N'), Key::Escape] {
        assert_eq!(review("a", "A.").on_key(key, 10), Action::KeepOriginal);
    }
    assert_eq!(
        review("a", "A.").on_key(Key::Char('x'), 10),
        Action::Continue
    );
}

#[test]
fn enter_while_viewing_the_original_still_accepts_the_edit() {
    let mut view = review("a", "A.");
    view.on_key(Key::Char('o'), 10);
    assert_eq!(view.on_key(Key::Enter, 10), Action::Accept("A.".into()));
}

#[test]
fn while_waiting_only_escape_and_n_leave() {
    assert_eq!(waiting("a").on_key(Key::Escape, 10), Action::KeepOriginal);
    assert_eq!(
        waiting("a").on_key(Key::Char('n'), 10),
        Action::KeepOriginal
    );
    assert_eq!(waiting("a").on_key(Key::Enter, 10), Action::Continue);
    assert_eq!(waiting("a").on_key(Key::Char('x'), 10), Action::Continue);
}

#[test]
fn a_failure_shows_the_message_and_any_key_keeps_the_text() {
    let mut view = View::new(
        "my text".into(),
        PROVIDER.into(),
        Phase::Failed("API returned HTTP 401.".into()),
    );
    let text = plain(&view.render(80, 12));
    assert!(text[0].contains("Could not edit"), "{}", text[0]);
    assert!(text.iter().any(|l| l.contains("API returned HTTP 401.")));
    assert!(text.iter().any(|l| l.contains("my text")));
    for key in [Key::Enter, Key::Escape, Key::Char('z')] {
        let mut again = view.clone();
        assert_eq!(again.on_key(key, 10), Action::KeepOriginal);
    }
}

#[test]
fn a_refused_text_is_never_shown_again_and_s_sends_it_anyway() {
    // Written in two pieces so no line here looks like a real key to a secret scanner.
    let secret = concat!("export KEY=sk-", "abcdefghijklmnopqrstuvwxyz");
    let mut view = View::new(
        secret.into(),
        PROVIDER.into(),
        Phase::Refused("an API key or token"),
    );
    let text = plain(&view.render(80, 12));
    let everything = text.join("\n");
    assert!(everything.contains("an API key or token"));
    assert!(everything.contains(PROVIDER));
    assert!(
        !everything.contains("sk-abcdef"),
        "the secret must not be echoed"
    );
    assert_eq!(view.on_key(Key::Char('s'), 10), Action::SendAnyway);
    assert_eq!(view.clone().on_key(Key::Char('S'), 10), Action::SendAnyway);
    for key in [Key::Enter, Key::Escape, Key::Char('n'), Key::Char('x')] {
        assert_eq!(view.clone().on_key(key, 10), Action::KeepOriginal);
    }
}

#[test]
fn long_text_wraps_and_scrolls() {
    let text: String = (1..=40).map(|n| format!("line {n}\n")).collect();
    let mut view = review(&text, &text);
    let top = plain(&view.render(60, 10));
    assert!(top.iter().any(|l| l.contains("line 1")));
    assert!(!top.iter().any(|l| l.contains("line 40")));
    for _ in 0..100 {
        view.on_key(Key::Down, 5);
    }
    let bottom = plain(&view.render(60, 10));
    assert!(bottom.iter().any(|l| l.contains("line 40")), "{bottom:?}");
    assert!(!bottom
        .iter()
        .any(|l| l.contains("line 1\n") || *l == "line 1"));
    view.on_key(Key::PageUp, 5);
    assert!(view.scroll < 100);
    for _ in 0..100 {
        view.on_key(Key::Up, 5);
    }
    assert!(plain(&view.render(60, 10))
        .iter()
        .any(|l| l.contains("line 1")));
}

#[test]
fn a_long_line_wraps_at_the_width_preferring_spaces() {
    let mut view = waiting("alpha beta gamma delta epsilon");
    let text = plain(&view.render(16, 10));
    let body: Vec<&String> = text[2..text.len() - 1]
        .iter()
        .filter(|l| !l.is_empty())
        .collect();
    assert_eq!(body, ["alpha beta ", "gamma delta ", "epsilon"]);
}

#[test]
fn control_characters_in_the_users_text_are_not_sent_to_the_terminal() {
    let mut view = waiting("a\u{1b}[31mb\u{7}c\td");
    let lines = view.render(80, 8);
    let everything: String = plain(&lines).join("\n");
    assert!(
        !everything.contains('\u{1b}') && !everything.contains('\u{7}'),
        "{everything:?}"
    );
    assert!(!everything.contains('\t'));
}
