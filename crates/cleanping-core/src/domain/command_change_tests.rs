use super::*;
use ChangeKind::*;

const FENCE: &str = "```";

fn kinds(original: &str, reply: &str) -> Vec<ChangeKind> {
    command_changes(original, reply)
        .into_iter()
        .map(|c| c.kind)
        .collect()
}

fn fenced(body: &str) -> String {
    format!("Why does this hang?\n{FENCE}sh\n{body}\n{FENCE}\nThanks.")
}

#[test]
fn the_table_of_cases() {
    let flag = |s: &str| FlagRemoved(s.into());
    let cases: Vec<(&str, String, String, Vec<ChangeKind>)> = vec![
        (
            "unclosed quote closed",
            fenced("curl -d '{\"name\": \"lamp\"}"),
            fenced("curl -d '{\"name\": \"lamp\"}'"),
            vec![QuoteClosed],
        ),
        (
            "balanced quote opened",
            fenced("echo 'a b'"),
            fenced("echo 'a b"),
            vec![QuoteOpened],
        ),
        (
            "flag dropped",
            fenced("rm -r -f build/"),
            fenced("rm -r build/"),
            vec![flag("-f")],
        ),
        (
            "flag added",
            fenced("ls /var/log"),
            fenced("ls -la /var/log"),
            vec![FlagAdded("-la".into())],
        ),
        (
            "path changed",
            fenced("cp notes.txt ./backup/"),
            fenced("cp notes.txt ./backups/"),
            vec![
                PathRemoved("./backup/".into()),
                PathAdded("./backups/".into()),
            ],
        ),
        (
            "prose apostrophes untouched",
            "i dont know why its broken, it's weird".into(),
            "I don't know why it's broken; it's weird.".into(),
            vec![],
        ),
        (
            "fenced block left identical",
            fenced("grep -rn 'todo' src/"),
            fenced("grep -rn 'todo' src/"),
            vec![],
        ),
        (
            "spelling fixed around an identical command",
            "pls chek why\n$ git push -u origin main\nfails wiht 403".into(),
            "Please check why\n$ git push -u origin main\nfails with 403.".into(),
            vec![],
        ),
        (
            "prose lead-in fixed on a line with a flag",
            "pls run: rm -r -f build/".into(),
            "Please run: rm -r build/".into(),
            vec![flag("-f")],
        ),
        ("empty inputs", String::new(), String::new(), vec![]),
        ("empty reply", fenced("ls -a"), String::new(), vec![Missing]),
        (
            "nested quotes",
            fenced("echo '\"hi\" she said'"),
            fenced("echo '\"hi\" she said"),
            vec![QuoteOpened],
        ),
        (
            "escaped quotes are not quotes",
            fenced("echo it\\'s \"a \\\"b\\\"\""),
            fenced("echo it\\'s \"a \\\"b\\\"\""),
            vec![],
        ),
        (
            "prompt line with an unclosed quote",
            "$ git commit -m \"fix login\nwhy does it wait".into(),
            "$ git commit -m \"fix login\"\nWhy does it wait?".into(),
            vec![QuoteClosed],
        ),
        (
            "prompt line and its continuation",
            "$ echo 'one\n> two'".into(),
            "$ echo 'one\n> two'".into(),
            vec![],
        ),
        (
            "indented command",
            "Run this:\n\n    tar -xzf app.tgz -C /opt\n".into(),
            "Run this:\n\n    tar -xzf app.tgz\n".into(),
            vec![flag("-C"), PathRemoved("/opt".into())],
        ),
        (
            "indented prose is not a command",
            "    this is 'just prose".into(),
            "    This is 'just prose'.".into(),
            vec![],
        ),
        (
            "unicode",
            fenced("echo \"h\u{e9}llo w\u{f6}rld \u{65e5}\u{672c}"),
            fenced("echo \"h\u{e9}llo w\u{f6}rld \u{65e5}\u{672c}\""),
            vec![QuoteClosed],
        ),
    ];
    for (name, original, reply, expected) in cases {
        assert_eq!(kinds(&original, &reply), expected, "case: {name}");
    }
}

#[test]
fn a_changed_command_is_described_in_words() {
    let changes = command_changes(
        &fenced("curl -X POST -d '{\"a\": 1}"),
        &fenced("curl -X POST -d '{\"a\": 1}'"),
    );
    assert_eq!(
        changes[0].to_string(),
        "a quote was closed in the command starting `curl -X POST`"
    );
}

#[test]
fn very_long_lines_are_handled_and_shortened_for_display() {
    let long = format!("/srv/{}", "a".repeat(100_000));
    let original = fenced(&format!("cat {long} -n"));
    assert!(command_changes(&original, &original).is_empty());
    let changes = command_changes(&original, &fenced("cat /srv/x -n"));
    let PathRemoved(shown) = &changes[0].kind else {
        panic!("{changes:?}")
    };
    assert_eq!(shown.chars().count(), 40);
    assert!(shown.ends_with('\u{2026}'));
    assert!(changes[0].to_string().chars().count() < 160);
}

#[test]
fn a_secret_is_never_shown() {
    let key = concat!("sk-", "Zx81QmVt3LpRw92NcYb47HdKe06Fa5Ug");
    let original = fenced(&format!("tool --key={key} run"));
    let changes = command_changes(&original, &fenced("tool run"));
    assert_eq!(changes.len(), 1, "{changes:?}");
    let shown = changes[0].to_string();
    assert!(!shown.contains("Zx81"), "{shown}");
    assert!(shown.contains("secret"), "{shown}");
}

#[test]
fn the_whole_line_counts_as_a_command_on_a_command_line() {
    let typed = "curl -d '{\"name\": \"lamp\"}";
    let fixed = "curl -d '{\"name\": \"lamp\"}'";
    let kinds: Vec<_> = command_line_changes(typed, fixed)
        .into_iter()
        .map(|c| c.kind)
        .collect();
    assert_eq!(kinds, vec![QuoteClosed]);
    assert!(command_line_changes("pleae fix this", "Please fix this.").is_empty());
    assert!(command_line_changes("i dont know", "I don't know.").is_empty());
    assert!(command_line_changes("", "").is_empty());
}

#[test]
fn a_summary_names_the_first_change_and_counts_the_rest() {
    assert_eq!(summary(&[]), None);
    let changes = command_changes(&fenced("rm -r -f x/"), &fenced("rm y/"));
    let text = summary(&changes).unwrap();
    assert!(text.starts_with("the flag `-r` was removed"), "{text}");
    assert!(text.ends_with("(and 3 more)"), "{text}");
}

/// The case that started this: a pasted command with its closing quote missing, and the author
/// asking why it waits. Every model tried closed the quote, which hides the answer.
#[test]
fn the_t9_example_is_caught() {
    let original = "why is it waiting?\ncurl -X POST http://localhost:8080/items -H \"Content-Type: application/json\" -d '{\"name\": \"lamp\"}";
    let reply = "Why is it waiting?\ncurl -X POST http://localhost:8080/items -H \"Content-Type: application/json\" -d '{\"name\": \"lamp\"}'";
    let changes = command_changes(original, reply);
    let said: Vec<String> = changes.iter().map(ToString::to_string).collect();
    assert_eq!(
        said,
        vec!["a quote was closed in the command starting `curl -X POST`"]
    );
    let line = command_line_changes(
        original.lines().nth(1).unwrap(),
        reply.lines().nth(1).unwrap(),
    );
    assert_eq!(line.len(), 1, "{line:?}");
}

#[test]
fn control_characters_in_a_command_are_never_shown() {
    let changes = command_changes(&fenced("echo \u{1b}[2J 'x"), &fenced("echo \u{1b}[2J 'x'"));
    let said = changes[0].to_string();
    assert!(!said.chars().any(char::is_control), "{said:?}");
}

/// Known false positives, kept here so the docs that list them stay true.
#[test]
fn known_false_positives() {
    let opened: Vec<_> = command_line_changes("the users files", "The users' files.")
        .into_iter()
        .map(|c| c.kind)
        .collect();
    assert_eq!(opened, vec![QuoteOpened], "an apostrophe at a word's edge");
    assert_eq!(
        kinds(&fenced("ls -la"), "Run `ls -la` to see them."),
        vec![Missing],
        "a command moved into a sentence in backticks"
    );
    assert!(
        kinds(&fenced("ls -la"), "Run ls -la to see them.").is_empty(),
        "without backticks the words in common still find it"
    );
}
