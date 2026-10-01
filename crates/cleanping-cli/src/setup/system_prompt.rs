//! Step 2: choose how the AI should edit the text.

use cleanping_core::domain::errors::Result;
use cleanping_core::domain::prompt_presets::{is_earlier_text, preset_for_body, PROMPT_PRESETS};

use super::ask::{choose, confirm};
use super::console::Console;
use crate::services::Services;

pub fn run(console: &mut dyn Console, services: &Services) -> Result<()> {
    let current = services.prompts.current()?;
    let in_use = preset_for_body(&current);
    let now_mark = if is_earlier_text(&current) {
        " (now, earlier version)"
    } else {
        " (now)"
    };
    let mut options: Vec<String> = PROMPT_PRESETS
        .iter()
        .map(|preset| {
            let now = if in_use == Some(preset) { now_mark } else { "" };
            format!("{:<9} {}{now}", preset.name, preset.description)
        })
        .collect();
    options.push(if in_use.is_some() {
        "Keep what I have now".into()
    } else {
        "Keep my own prompt (now)".into()
    });
    let keep = options.len() - 1;
    let title = "Step 2: how should the AI edit your text?";
    let index = choose(console, title, &options, Some(keep))?;
    if index == keep {
        console.say("Keeping the current system prompt.");
    } else {
        let preset = &PROMPT_PRESETS[index];
        let replaces_your_own = in_use.is_none();
        let question = "That replaces the system prompt you wrote yourself. Replace it?";
        if replaces_your_own && !confirm(console, question, false)? {
            console.say("Keeping your own system prompt.");
        } else {
            services.prompts.save(preset.body)?;
            console.say(&format!(
                "System prompt set to the \u{201c}{}\u{201d} preset.",
                preset.name
            ));
        }
    }
    console.say("To write your own any time: cleanping prompt set \"...\"");
    Ok(())
}

#[cfg(test)]
#[path = "system_prompt_tests.rs"]
mod tests;
