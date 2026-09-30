use super::*;

fn commands(text: &str) -> Vec<String> {
    pieces(text)
        .into_iter()
        .filter(|p| p.command)
        .map(|p| p.text)
        .collect()
}

#[test]
fn every_line_in_a_fence_is_a_command_and_the_fence_lines_are_not() {
    let text = "look:\n```sh\nmake\n$ make test\n```\nok";
    assert_eq!(commands(text), ["make", "make test"]);
}

#[test]
fn a_prompt_marker_makes_a_command_and_is_removed() {
    assert_eq!(commands("  $ ls\nfile.txt"), ["ls"]);
}

#[test]
fn an_indented_line_is_a_command_only_when_it_looks_like_one() {
    assert_eq!(commands("    cat /etc/hosts"), ["cat /etc/hosts"]);
    assert_eq!(commands("\tgrep -n x"), ["grep -n x"]);
    assert!(commands("    and then we went home").is_empty());
    assert!(commands("    - a list item /with/a/path").is_empty());
}

#[test]
fn an_unindented_line_needs_a_flag() {
    assert_eq!(commands("curl -X POST http://x"), ["curl -X POST http://x"]);
    assert!(commands("cat /etc/hosts").is_empty());
    assert!(commands("It's a -- dash and a - dash").is_empty());
    assert!(commands("Why is it waiting?").is_empty());
}

#[test]
fn prose_with_apostrophes_is_not_a_command() {
    assert!(commands("I don't know why it's broken, isn't it?").is_empty());
    assert!(commands("    it's what we don't do").is_empty());
}

#[test]
fn continuation_lines_join_after_a_backslash_or_an_open_quote() {
    let text = "$ echo 'one\n> two'\n> not this one\n$ tar -c \\\n> -f x.tar";
    assert_eq!(commands(text), ["echo 'one\ntwo'", "tar -c \\\n-f x.tar"]);
}

#[test]
fn outside_a_fence_only_a_continuation_prompt_continues() {
    assert_eq!(commands("curl -d 'x\nwhy does it wait?"), ["curl -d 'x"]);
}

#[test]
fn inside_a_fence_an_open_quote_takes_the_next_lines_until_the_fence_ends() {
    let text = "```\necho 'a\nb\n```\nafter";
    assert_eq!(commands(text), ["echo 'a\nb"]);
}

#[test]
fn other_lines_are_kept_as_plain_pieces() {
    let all = pieces("hello\n\n$ ls -a\n  bye  ");
    let plain: Vec<_> = all
        .iter()
        .filter(|p| !p.command)
        .map(|p| &p.text[..])
        .collect();
    assert_eq!(plain, ["hello", "bye"]);
}

#[test]
fn flags_and_paths_ignore_sentence_punctuation_and_quotes() {
    let words = scan("x -v, --out=a /tmp/log. '-q' '/etc' - -- ~/x a/b").words;
    let flags: Vec<_> = words.iter().filter_map(flag).collect();
    let paths: Vec<_> = words.iter().filter_map(path).collect();
    assert_eq!(flags, ["-v", "--out=a"]);
    assert_eq!(paths, ["/tmp/log", "~/x", "a/b"]);
}
