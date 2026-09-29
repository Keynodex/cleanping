//! Which words of an edited text differ from the original, for highlighting.

/// A piece of the edited text and whether it is new or different compared to the original.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    pub text: String,
    pub changed: bool,
}

/// The edited text cut into pieces, in order. Joining every `text` gives the edited text back
/// exactly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diff {
    pub pieces: Vec<Piece>,
    /// Words of the original that no longer appear (they have no place in the edited text).
    pub removed_words: usize,
}

/// Compare word by word. `None` when the texts are too large to compare quickly; the caller
/// then shows the edited text without highlighting.
pub fn highlight_changes(original: &str, edited: &str) -> Option<Diff> {
    let before: Vec<&str> = original.split_whitespace().collect();
    let tokens = tokenize(edited);
    let after: Vec<&str> = tokens
        .iter()
        .filter(|t| t.is_word)
        .map(|t| t.text)
        .collect();
    let matched = match_words(&before, &after)?;
    let kept = matched.iter().filter(|&&m| m).count();
    Some(Diff {
        pieces: build_pieces(&tokens, &matched),
        removed_words: before.len() - kept,
    })
}

/// The most cells the comparison table may have; beyond it the texts are not compared.
const MAX_CELLS: usize = 4_000_000;

struct Token<'a> {
    text: &'a str,
    is_word: bool,
}

/// Alternating runs of blanks and of words, covering the text exactly.
fn tokenize(text: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut start = 0;
    let mut current: Option<bool> = None;
    for (index, c) in text.char_indices() {
        let is_word = !c.is_whitespace();
        if current.is_some_and(|previous| previous != is_word) {
            tokens.push(Token {
                text: &text[start..index],
                is_word: !is_word,
            });
            start = index;
        }
        current = Some(is_word);
    }
    if let Some(is_word) = current {
        tokens.push(Token {
            text: &text[start..],
            is_word,
        });
    }
    tokens
}

/// For each word of `after`, whether it lines up with a word of `before` (longest common
/// subsequence). `None` when the table would be too big.
fn match_words(before: &[&str], after: &[&str]) -> Option<Vec<bool>> {
    let mut matched = vec![false; after.len()];
    let prefix = before.iter().zip(after).take_while(|(a, b)| a == b).count();
    matched[..prefix].fill(true);
    let max_suffix = before.len().min(after.len()) - prefix;
    let suffix = before
        .iter()
        .rev()
        .zip(after.iter().rev())
        .take(max_suffix)
        .take_while(|(a, b)| a == b)
        .count();
    let (a, b) = (
        &before[prefix..before.len() - suffix],
        &after[prefix..after.len() - suffix],
    );
    matched[after.len() - suffix..].fill(true);
    if a.is_empty() || b.is_empty() {
        return Some(matched);
    }
    if (a.len() + 1).checked_mul(b.len() + 1)? > MAX_CELLS {
        return None;
    }
    let width = b.len() + 1;
    let mut table = vec![0u32; (a.len() + 1) * width];
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            table[i * width + j] = if a[i - 1] == b[j - 1] {
                table[(i - 1) * width + j - 1] + 1
            } else {
                table[(i - 1) * width + j].max(table[i * width + j - 1])
            };
        }
    }
    let (mut i, mut j) = (a.len(), b.len());
    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            matched[prefix + j - 1] = true;
            i -= 1;
            j -= 1;
        } else if table[(i - 1) * width + j] >= table[i * width + j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    Some(matched)
}

/// Mark new words, also mark the blanks between two new words, and merge neighbours.
fn build_pieces(tokens: &[Token<'_>], matched: &[bool]) -> Vec<Piece> {
    let mut word = 0;
    let mut changed: Vec<bool> = tokens
        .iter()
        .map(|t| {
            if !t.is_word {
                return false;
            }
            word += 1;
            !matched[word - 1]
        })
        .collect();
    for i in 1..tokens.len().saturating_sub(1) {
        if !tokens[i].is_word && changed[i - 1] && changed[i + 1] {
            changed[i] = true;
        }
    }
    let mut pieces: Vec<Piece> = Vec::new();
    for (token, is_changed) in tokens.iter().zip(changed) {
        match pieces.last_mut() {
            Some(last) if last.changed == is_changed => last.text.push_str(token.text),
            _ => pieces.push(Piece {
                text: token.text.to_string(),
                changed: is_changed,
            }),
        }
    }
    pieces
}

#[cfg(test)]
#[path = "diff_tests.rs"]
mod tests;
