//! Spotting text that looks like it holds a secret, so it is not sent to an AI provider by
//! accident. A best-effort guard against mistakes, not a security boundary.

/// What kind of secret the text seems to contain, or `None`. Never returns the secret itself.
pub fn find_secret(text: &str) -> Option<&'static str> {
    if text.contains("-----BEGIN") && text.contains("PRIVATE KEY-----") {
        return Some("a private key");
    }
    let lower = text.to_ascii_lowercase();
    if has_key_shape(text) || has_bearer_token(&lower) {
        return Some("an API key or token");
    }
    if has_assigned_secret(text, &lower) {
        return Some("a password or secret");
    }
    None
}

/// (prefix, characters that may follow it, how many must follow) for common API key formats.
type KeyShape = (&'static str, fn(char) -> bool, usize);

const KEY_SHAPES: &[KeyShape] = &[
    ("sk-", word_char, 20),
    ("sk_live_", word_char, 16),
    ("rk_live_", word_char, 16),
    ("ghp_", word_char, 30),
    ("gho_", word_char, 30),
    ("ghu_", word_char, 30),
    ("ghs_", word_char, 30),
    ("ghr_", word_char, 30),
    ("github_pat_", word_char, 20),
    ("glpat-", word_char, 20),
    ("xoxb-", word_char, 20),
    ("xoxp-", word_char, 20),
    ("xoxa-", word_char, 20),
    ("AKIA", upper_digit, 16),
    ("ASIA", upper_digit, 16),
    ("AIza", word_char, 35),
    ("hf_", word_char, 30),
];

fn word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

fn upper_digit(c: char) -> bool {
    c.is_ascii_uppercase() || c.is_ascii_digit()
}

/// A known prefix that starts a word and is followed by enough key-like characters.
fn has_key_shape(text: &str) -> bool {
    KEY_SHAPES.iter().any(|(prefix, allowed, needed)| {
        text.match_indices(prefix).any(|(at, _)| {
            let starts_word = text[..at].chars().next_back().is_none_or(|c| !word_char(c));
            starts_word
                && text[at + prefix.len()..]
                    .chars()
                    .take_while(|&c| allowed(c))
                    .count()
                    >= *needed
        })
    })
}

/// `Bearer` followed by a long token, as in an Authorization header.
fn has_bearer_token(lower: &str) -> bool {
    lower.match_indices("bearer ").any(|(at, word)| {
        lower[at + word.len()..]
            .chars()
            .take_while(|&c| c.is_ascii_alphanumeric() || "._~+/=-".contains(c))
            .count()
            >= 20
    })
}

const SECRET_NAMES: &[&str] = &[
    "password",
    "passwd",
    "secret",
    "token",
    "api_key",
    "apikey",
    "api-key",
    "access_key",
    "private_key",
];

/// `name=value` or `name: value` where the name says secret and the value looks like one.
fn has_assigned_secret(text: &str, lower: &str) -> bool {
    SECRET_NAMES.iter().any(|name| {
        lower.match_indices(name).any(|(at, _)| {
            let rest = text[at + name.len()..].trim_start_matches(['"', '\'']);
            let rest = rest.trim_start_matches([' ', '\t']);
            let Some(rest) = rest.strip_prefix(['=', ':']) else {
                return false;
            };
            let value: String = rest
                .trim_start_matches([' ', '\t', '"', '\''])
                .chars()
                .take_while(|c| !c.is_whitespace() && !"\"',;})".contains(*c))
                .collect();
            value.chars().count() >= 6 && looks_chosen(&value)
        })
    })
}

/// Plain lowercase words ("please") are prose; digits, symbols or mixed case look chosen.
fn looks_chosen(value: &str) -> bool {
    value
        .chars()
        .any(|c| c.is_ascii_digit() || "!@#$%^&*+/=_".contains(c))
        || (value.chars().any(char::is_lowercase) && value.chars().any(char::is_uppercase))
}

#[cfg(test)]
#[path = "secret_scan_tests.rs"]
mod tests;
