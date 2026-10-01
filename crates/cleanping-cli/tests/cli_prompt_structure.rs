//! The `structure` ready-made system prompt, through the real binary.

mod support;

use support::*;

/// The prompt file, byte for byte (it ends with a newline).
const STRUCTURE_FILE: &str = include_str!("../../cleanping-core/src/domain/prompts/structure.txt");

#[test]
fn prompt_presets_lists_structure_with_its_description_after_friendly() {
    let out = Sandbox::new().run(&["prompt", "presets"], None);
    assert_eq!(out.code, 0, "{}", out.stderr);
    let names: Vec<&str> = out
        .stdout
        .lines()
        .filter_map(|l| l.get(2..)?.split_whitespace().next())
        .collect();
    let friendly = names.iter().position(|n| *n == "friendly").unwrap();
    assert_eq!(
        names.get(friendly + 1),
        Some(&"structure"),
        "{}",
        out.stdout
    );
    assert!(
        out.stdout
            .lines()
            .any(|l| l
                == "  structure Fix and lay out as a clear AI prompt; clean pasted terminal junk"),
        "{}",
        out.stdout
    );
}

#[test]
fn prompt_use_structure_saves_the_file_text_exactly_and_marks_it() {
    let sandbox = Sandbox::new();
    let used = sandbox.run(&["prompt", "use", "STRUCTURE"], None);
    assert_eq!(used.code, 0, "{}", used.stderr);
    assert!(
        used.stdout.contains("\u{201c}structure\u{201d}"),
        "{}",
        used.stdout
    );
    let shown = sandbox.run(&["prompt", "show"], None);
    assert_eq!(shown.stdout, STRUCTURE_FILE);
    let listed = sandbox.run(&["prompt", "presets"], None).stdout;
    let marked: Vec<&str> = listed.lines().filter(|l| l.starts_with('*')).collect();
    assert_eq!(marked.len(), 1, "{listed}");
    assert!(marked[0].starts_with("* structure "), "{listed}");
}

#[test]
fn a_rewrite_sends_the_structure_prompt_to_the_provider() {
    let server = serve(vec![ok_reply("Fix the build.")]);
    let sandbox = Sandbox::new();
    sandbox.add_local("Local", &server);
    assert_eq!(sandbox.run(&["prompt", "use", "structure"], None).code, 0);
    let out = sandbox.run(&["fix teh build"], None);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert_eq!(server.saved_prompt(0), STRUCTURE_FILE.trim_end());
    assert_eq!(server.user_text(0), "fix teh build");
}
