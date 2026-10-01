use super::*;

fn texts(command: &str) -> Vec<String> {
    scan(command).words.into_iter().map(|w| w.text).collect()
}

#[test]
fn words_split_on_blanks_outside_quotes() {
    assert_eq!(texts("ls  -la\t/tmp"), ["ls", "-la", "/tmp"]);
    assert_eq!(
        texts("curl -d '{\"a\": 1}' x"),
        ["curl", "-d", "'{\"a\": 1}'", "x"]
    );
    assert_eq!(texts(""), Vec::<String>::new());
}

#[test]
fn an_unclosed_quote_is_reported_and_a_closed_one_is_not() {
    assert_eq!(scan("echo 'a b").open, Some('\''));
    assert_eq!(scan("echo \"a b").open, Some('"'));
    assert_eq!(scan("echo `date").open, Some('`'));
    assert_eq!(scan("echo 'a b'").open, None);
}

#[test]
fn quotes_inside_other_quotes_are_plain_characters() {
    assert_eq!(scan("echo '\"'").open, None);
    assert_eq!(scan("git commit -m \"it's done\"").open, None);
    assert_eq!(scan("echo \"a 'b\"").open, None);
}

#[test]
fn a_backslash_escapes_a_quote_outside_single_quotes() {
    assert_eq!(scan("echo it\\'s").open, None);
    assert_eq!(scan("echo \"say \\\"hi\\\"\"").open, None);
    // Inside single quotes a backslash is a plain character, so this quote closes.
    assert_eq!(scan("echo 'a\\'").open, None);
}

#[test]
fn an_apostrophe_between_letters_is_not_a_quote() {
    assert_eq!(scan("i don't know").open, None);
    // A quote at the start of a word is still a quote.
    assert_eq!(scan("echo 'its").open, Some('\''));
    assert_eq!(scan("echo x '").open, Some('\''));
}

#[test]
fn a_comment_hides_its_quotes() {
    assert_eq!(scan("ls # can't").open, None);
    assert_eq!(texts("ls -a # -b"), ["ls", "-a"]);
    assert_eq!(texts("a#b"), ["a#b"]);
}

#[test]
fn a_backslash_newline_joins_lines_and_a_trailing_one_continues() {
    assert_eq!(texts("ls \\\n-la"), ["ls", "-la"]);
    let mut scanner = Scanner::default();
    scanner.feed("tar -c \\");
    assert!(scanner.continues());
    scanner.feed("\n-f x.tar");
    assert!(!scanner.continues());
    assert_eq!(
        scanner
            .finish()
            .words
            .iter()
            .map(|w| &w.text[..])
            .collect::<Vec<_>>(),
        ["tar", "-c", "-f", "x.tar"]
    );
}

#[test]
fn a_word_that_starts_with_a_quote_is_marked_quoted() {
    let words = scan("say \"-x\" -y").words;
    assert!(words[1].quoted && !words[2].quoted, "{words:?}");
}
