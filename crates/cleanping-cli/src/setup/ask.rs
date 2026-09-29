//! The two questions the wizard keeps asking: pick a number, and yes or no.

use cleanping_core::domain::errors::Result;

use super::console::{too_many_bad_answers, Console};

/// Bad answers in a row before the wizard gives up instead of asking forever.
const TRIES: usize = 5;

/// Show `options` numbered from 1 and return the 0-based index of the one picked. `default`
/// (0-based) is what Enter alone picks.
pub fn choose(
    console: &mut dyn Console,
    title: &str,
    options: &[String],
    default: Option<usize>,
) -> Result<usize> {
    console.say(title);
    for (index, option) in options.iter().enumerate() {
        console.say(&format!("  {}) {option}", index + 1));
    }
    let count = options.len();
    let question = match default {
        Some(index) => format!("Number (1-{count}, Enter = {}): ", index + 1),
        None => format!("Number (1-{count}): "),
    };
    for _ in 0..TRIES {
        let answer = console.ask(&question)?;
        let answer = answer.trim();
        if answer.is_empty() {
            if let Some(index) = default {
                return Ok(index);
            }
        } else if let Ok(number) = answer.parse::<usize>() {
            if (1..=count).contains(&number) {
                return Ok(number - 1);
            }
        }
        console.say(&format!("Please type a number from 1 to {count}."));
    }
    Err(too_many_bad_answers())
}

/// Ask a yes/no question; Enter alone gives `default_yes`.
pub fn confirm(console: &mut dyn Console, question: &str, default_yes: bool) -> Result<bool> {
    let hint = if default_yes { "[Y/n]" } else { "[y/N]" };
    for _ in 0..TRIES {
        let answer = console.ask(&format!("{question} {hint} "))?;
        match answer.trim().to_ascii_lowercase().as_str() {
            "" => return Ok(default_yes),
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => console.say("Please answer y or n."),
        }
    }
    Err(too_many_bad_answers())
}

#[cfg(test)]
#[path = "ask_tests.rs"]
mod tests;
