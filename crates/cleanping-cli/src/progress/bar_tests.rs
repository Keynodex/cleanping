use super::*;

const UNICODE: Look = Look {
    unicode: true,
    color: false,
};
const ASCII: Look = Look {
    unicode: false,
    color: false,
};

fn line(percent: u8, secs: u64, width: usize, look: Look) -> String {
    render_bar(Progress { percent }, Duration::from_secs(secs), width, look)
}

fn columns(text: &str) -> usize {
    text.chars().count()
}

#[test]
fn a_wide_terminal_gets_the_words_a_bar_of_twenty_and_the_estimate() {
    assert_eq!(
        line(40, 12, 80, UNICODE),
        "Fixing your text\u{2026} \u{25b0}\u{25b0}\u{25b0}\u{25b0}\u{25b0}\u{25b0}\u{25b0}\u{25b0}\
         \u{25b1}\u{25b1}\u{25b1}\u{25b1}\u{25b1}\u{25b1}\u{25b1}\u{25b1}\u{25b1}\u{25b1}\u{25b1}\u{25b1} \
         about 40% 12s"
    );
}

#[test]
fn forty_columns_keep_the_words_and_a_shorter_bar() {
    let text = line(50, 7, 40, UNICODE);
    assert!(
        text.starts_with("Fixing your text\u{2026} \u{25b0}"),
        "{text}"
    );
    assert!(text.ends_with(" about 50% 7s"), "{text}");
    assert!(columns(&text) < 40, "{text}");
}

#[test]
fn twenty_columns_drop_the_words_but_keep_a_bar_and_the_numbers() {
    let text = line(50, 7, 20, UNICODE);
    assert!(text.starts_with('\u{25b0}'), "{text}");
    assert!(text.ends_with(" about 50% 7s"), "{text}");
    assert!(columns(&text) < 20, "{text}");
}

#[test]
fn every_width_leaves_the_last_column_free_even_after_minutes() {
    for width in 0..=120 {
        for look in [UNICODE, ASCII] {
            let text = line(95, 185, width, look);
            assert!(columns(&text) < width.max(1), "{width}: {text}");
        }
    }
}

#[test]
fn the_bar_keeps_its_length_as_the_seconds_grow() {
    let blocks = |secs| {
        line(40, secs, 40, UNICODE)
            .chars()
            .filter(|c| matches!(c, '\u{25b0}' | '\u{25b1}'))
            .count()
    };
    assert_eq!(blocks(5), blocks(150));
}

#[test]
fn zero_percent_is_an_empty_bar() {
    let text = line(0, 1, 80, UNICODE);
    assert!(!text.contains('\u{25b0}'), "{text}");
    assert!(text.contains(&"\u{25b1}".repeat(20)), "{text}");
    assert!(text.ends_with(" about 0% 1s"), "{text}");
}

#[test]
fn ninety_five_percent_leaves_one_block_empty() {
    let text = line(95, 30, 80, UNICODE);
    assert!(
        text.contains(&format!("{}\u{25b1} ", "\u{25b0}".repeat(19))),
        "{text}"
    );
    assert!(text.ends_with(" about 95% 30s"), "{text}");
}

#[test]
fn ascii_uses_hashes_dots_and_brackets() {
    assert_eq!(
        line(40, 12, 80, ASCII),
        "Fixing your text... [########............] about 40% 12s"
    );
    let narrow = line(40, 12, 40, ASCII);
    assert!(narrow.is_ascii(), "{narrow}");
    assert!(narrow.starts_with("[#") && narrow.ends_with("] about 40% 12s"));
}

#[test]
fn only_the_filled_blocks_are_green_and_only_when_color_is_allowed() {
    let color = Look {
        unicode: true,
        color: true,
    };
    let text = line(10, 3, 80, color);
    assert!(
        text.contains("\u{1b}[32m\u{25b0}\u{25b0}\u{1b}[0m\u{25b1}"),
        "{text:?}"
    );
    assert!(!line(10, 3, 80, UNICODE).contains('\u{1b}'));
    assert!(
        !line(0, 3, 80, color).contains('\u{1b}'),
        "no blocks, no color"
    );
}

#[test]
fn with_no_room_for_a_bar_the_numbers_remain() {
    assert_eq!(line(40, 12, 16, UNICODE), "about 40% 12s");
    assert_eq!(line(40, 12, 5, UNICODE), "abou");
}
