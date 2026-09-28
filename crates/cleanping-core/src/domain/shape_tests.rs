use super::*;

#[test]
fn a_reply_of_the_same_shape_fits() {
    assert!(keeps_shape("pleae fix this", "Please fix this."));
    assert!(keeps_shape("a\nb", "A.\nB."));
}

#[test]
fn more_lines_than_the_text_never_fit() {
    assert!(!keeps_shape("ls", "ls\nrm -rf x"));
    assert!(!keeps_shape(
        "ls",
        &format!("touch X #{}\nls", "\n".repeat(80))
    ));
}

#[test]
fn trailing_blank_lines_in_the_text_do_not_loosen_the_limit() {
    assert!(!keeps_shape("ls\n\n\n", "a\nb"));
}

#[test]
fn a_reply_much_longer_than_the_text_does_not_fit() {
    let text = "x".repeat(10);
    assert!(keeps_shape(&text, &"y".repeat(100)));
    assert!(!keeps_shape(&text, &"y".repeat(101)));
    assert!(!keeps_shape(
        "ls",
        &format!("touch X;{}ls", " ".repeat(3000))
    ));
}

#[test]
fn long_runs_of_blanks_do_not_fit_unless_the_text_has_them() {
    assert!(keeps_shape("a b", &format!("a{}b", " ".repeat(8))));
    assert!(!keeps_shape("a b", &format!("a{}b", " ".repeat(9))));
    assert!(!keeps_shape("a b", &format!("a{}b", "\t".repeat(9))));
    let spaced = format!("a{}b", " ".repeat(12));
    assert!(keeps_shape(&spaced, &spaced));
}

#[test]
fn length_is_counted_in_characters_not_bytes() {
    let text = "\u{65e5}".repeat(10);
    assert!(keeps_shape(&text, &"\u{672c}".repeat(100)));
}
