use super::*;

#[test]
fn plain_text_is_unchanged() {
    assert_eq!(clean_reply("Please fix this."), "Please fix this.");
}

#[test]
fn keeps_newlines_tabs_and_real_world_unicode_including_emoji_joiners() {
    let text = "Line one\n\tLine two \u{2014} caf\u{e9} \u{65e5}\u{672c}\u{8a9e} \u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}";
    assert_eq!(clean_reply(text), text);
}

#[test]
fn strips_escape_sequences_bells_nul_and_del() {
    let hostile = "a\u{1b}]52;c;ZWNobw==\u{7}b\u{0}c\u{7f}d";
    assert_eq!(clean_reply(hostile), "a]52;c;ZWNobw==bcd");
}

#[test]
fn strips_c1_controls() {
    assert_eq!(clean_reply("x\u{9b}31my"), "x31my");
}

#[test]
fn strips_bidi_overrides_and_isolates() {
    assert_eq!(clean_reply("a\u{202e}b\u{2066}c\u{2069}d"), "abcd");
}

#[test]
fn keeps_the_directional_marks_real_right_to_left_text_needs() {
    assert_eq!(clean_reply("a\u{200f}b"), "a\u{200f}b");
}

#[test]
fn carriage_returns_become_newlines_so_text_cannot_overwrite_itself() {
    assert_eq!(clean_reply("good\r\nreal\rfake"), "good\nreal\nfake");
}

#[test]
fn unicode_line_separators_become_newlines() {
    assert_eq!(clean_reply("a\u{2028}b\u{2029}c"), "a\nb\nc");
}

#[test]
fn trims_the_ends_and_can_end_up_empty() {
    assert_eq!(clean_reply("  hi \n"), "hi");
    assert_eq!(clean_reply("\u{1b}\u{7}  "), "");
}

#[test]
fn cleaning_twice_changes_nothing() {
    let once = clean_reply("a\u{1b}[31m\r\nb\u{202e}");
    assert_eq!(clean_reply(&once), once);
}

#[test]
fn strips_invisible_characters_that_could_hide_text() {
    let hidden = [
        '\u{ad}',
        '\u{61c}',
        '\u{180e}',
        '\u{200b}',
        '\u{2060}',
        '\u{2062}',
        '\u{206a}',
        '\u{3164}',
        '\u{feff}',
        '\u{ffa0}',
        '\u{e0041}',
        '\u{e007f}',
    ];
    for c in hidden {
        assert_eq!(clean_reply(&format!("a{c}b")), "ab", "U+{:04X}", c as u32);
    }
}

#[test]
fn keeps_the_joiners_and_marks_that_real_scripts_need() {
    let persian = "\u{645}\u{6cc}\u{200c}\u{62e}\u{648}\u{627}\u{647}\u{645}";
    assert_eq!(clean_reply(persian), persian);
    assert_eq!(clean_reply("a\u{200e}b"), "a\u{200e}b");
}

#[test]
fn a_line_of_plain_text_is_plain_and_anything_else_is_not() {
    assert!(is_plain_line("Work key 2"));
    assert!(is_plain_line("caf\u{e9} \u{65e5}\u{672c}"));
    for bad in [
        "a\nb",
        "a\tb",
        "a\u{1b}[31m",
        "a\u{202e}b",
        "a\u{200b}b",
        "a\rb",
    ] {
        assert!(!is_plain_line(bad), "{bad:?}");
    }
}
