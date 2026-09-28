//! Whether a reply may stand in for the text it replaces on a command line.

/// A reply may not have more lines than the text, be much longer, or pad itself with long runs
/// of blanks. Otherwise its first part could scroll out of sight while the harmless-looking
/// tail is what you read before pressing Enter.
pub fn keeps_shape(text: &str, reply: &str) -> bool {
    let (text, reply) = (text.trim(), reply.trim());
    reply.lines().count() <= text.lines().count()
        && reply.chars().count() <= text.chars().count() * 2 + EXTRA_CHARS
        && longest_blank_run(reply) <= longest_blank_run(text).max(MAX_BLANK_RUN)
}

/// Room for a rewrite to say more than a terse note did.
const EXTRA_CHARS: usize = 80;
const MAX_BLANK_RUN: usize = 8;

fn longest_blank_run(text: &str) -> usize {
    text.split(|c| c != ' ' && c != '\t')
        .map(str::len)
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
#[path = "shape_tests.rs"]
mod tests;
