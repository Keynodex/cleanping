use super::*;

fn piece(text: &str, changed: bool) -> Piece {
    Piece {
        text: text.into(),
        changed,
    }
}

fn joined(diff: &Diff) -> String {
    diff.pieces.iter().map(|p| p.text.as_str()).collect()
}

#[test]
fn identical_text_has_nothing_highlighted() {
    let diff = highlight_changes("fix the login", "fix the login").unwrap();
    assert_eq!(diff.pieces, [piece("fix the login", false)]);
    assert_eq!(diff.removed_words, 0);
}

#[test]
fn a_replaced_word_is_the_only_highlight() {
    let diff = highlight_changes("fix teh login", "fix the login").unwrap();
    assert_eq!(
        diff.pieces,
        [
            piece("fix ", false),
            piece("the", true),
            piece(" login", false)
        ]
    );
}

#[test]
fn an_inserted_word_is_highlighted() {
    let diff = highlight_changes("hello world", "hello big world").unwrap();
    assert_eq!(
        diff.pieces,
        [
            piece("hello ", false),
            piece("big", true),
            piece(" world", false)
        ]
    );
}

#[test]
fn blanks_between_two_changed_words_are_highlighted_too() {
    let diff = highlight_changes("a b c d", "a x y d").unwrap();
    assert_eq!(
        diff.pieces,
        [piece("a ", false), piece("x y", true), piece(" d", false)]
    );
    assert_eq!(diff.removed_words, 2);
}

#[test]
fn punctuation_and_capitals_count_as_changes() {
    let diff = highlight_changes("pleae fix this", "Please fix this.").unwrap();
    assert_eq!(
        diff.pieces,
        [
            piece("Please", true),
            piece(" fix ", false),
            piece("this.", true)
        ]
    );
}

#[test]
fn removed_words_are_counted_but_have_nothing_to_highlight() {
    let diff = highlight_changes("a b c", "a c").unwrap();
    assert_eq!(diff.pieces, [piece("a c", false)]);
    assert_eq!(diff.removed_words, 1);
}

#[test]
fn an_empty_result_removes_everything() {
    let diff = highlight_changes("a b", "").unwrap();
    assert!(diff.pieces.is_empty());
    assert_eq!(diff.removed_words, 2);
}

#[test]
fn line_breaks_tabs_and_unicode_survive_exactly() {
    for (original, edited) in [
        ("l1\nl2\tx", "L1\nl2\tx y"),
        (
            "caf\u{e9} \u{65e5}\u{672c}",
            "caf\u{e9}s \u{65e5}\u{672c}\n",
        ),
        ("  lead", "lead  "),
        ("", "new text"),
    ] {
        let diff = highlight_changes(original, edited).unwrap();
        assert_eq!(joined(&diff), edited, "{original:?} -> {edited:?}");
    }
}

#[test]
fn deleting_a_middle_word_highlights_nothing() {
    let diff = highlight_changes("one two three four", "one three four").unwrap();
    assert!(diff.pieces.iter().all(|p| !p.changed));
}

#[test]
fn texts_too_large_to_compare_quickly_give_none() {
    let big_a = "alpha ".repeat(3000);
    let big_b = "beta ".repeat(3000);
    assert!(highlight_changes(&big_a, &big_b).is_none());
}

#[test]
fn large_but_mostly_equal_texts_still_work() {
    let base = "same ".repeat(5000);
    let edited = format!("{base}extra");
    let diff = highlight_changes(&base, &edited).unwrap();
    assert!(diff.pieces.last().unwrap().changed);
}

#[test]
fn changed_ranges_are_character_positions_in_the_edited_text() {
    let diff = highlight_changes("pleae fix this", "Please fix this.").unwrap();
    assert_eq!(diff.changed_ranges(), [(0, 6), (11, 16)]);
}

#[test]
fn nothing_changed_means_no_ranges() {
    let diff = highlight_changes("fix the login", "fix the login").unwrap();
    assert_eq!(diff.changed_ranges(), []);
}

#[test]
fn ranges_count_characters_not_bytes() {
    // "café" is 4 characters but 5 bytes; "naïve" 5 characters, 6 bytes.
    let diff = highlight_changes("cafe ok naive", "café ok naïve").unwrap();
    assert_eq!(diff.changed_ranges(), [(0, 4), (8, 13)]);
}

#[test]
fn ranges_follow_line_breaks_and_tabs() {
    let diff = highlight_changes("a b\nc d", "a B\n\tc D").unwrap();
    let text = joined(&diff);
    let chars: Vec<char> = text.chars().collect();
    let cut: Vec<String> = diff
        .changed_ranges()
        .iter()
        .map(|&(start, end)| chars[start..end].iter().collect())
        .collect();
    assert_eq!(cut, ["B", "D"]);
}
