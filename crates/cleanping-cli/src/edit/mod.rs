//! `cleanping edit FILE`: fix the text in a file with the AI and let the user review it.
//! Meant to be set as `$VISUAL`, so the editor key of Claude Code or Codex opens it.
//!
//! With a screen the user accepts (Enter) or keeps the original (N). With `--yes` the edited
//! text is written straight into the file. Text that looks like it holds a secret is never
//! sent without a deliberate choice on the screen, and never with `--yes`.

mod file;
mod job;
mod layout;
mod session;
mod terminal;
mod view;
mod wrap;

use cleanping_core::domain::errors::Result;
use cleanping_core::domain::models::Credential;
use cleanping_core::domain::secret_scan::find_secret;
use cleanping_core::domain::validation::host_of;

use crate::args::EditArgs;
use crate::guard;
use crate::rewrite::pick_credential;
use crate::services::Services;
use job::Job;
use session::Outcome;
use terminal::Screen;

/// For example `DeepSeek (api.deepseek.com)`: where the text is going.
fn provider_label(credential: &Credential) -> String {
    match host_of(&credential.api_url) {
        Some(host) => format!("{} ({host})", credential.name),
        None => credential.name.clone(),
    }
}

pub fn run(services: &Services, args: &EditArgs) -> Result<()> {
    let raw = file::read(&args.file)?;
    let text = raw.trim();
    if text.is_empty() {
        return Ok(());
    }
    let secret = find_secret(text);
    let credential = pick_credential(args.credential.as_deref(), services);
    if args.yes {
        return edit_quietly(services, args, &raw, secret, credential);
    }
    let mut screen = Screen::open()?;
    let provider = credential
        .as_ref()
        .map_or_else(|_| "your provider".into(), provider_label);
    let prepared = credential.and_then(|credential| Job::prepare(services, text, credential));
    let outcome = session::run(&mut screen, text, provider, secret, prepared)?;
    drop(screen);
    match outcome {
        Outcome::Accept(edited) => file::write(&args.file, &file::replacement(&raw, &edited)),
        Outcome::Keep => Ok(()),
    }
}

fn edit_quietly(
    services: &Services,
    args: &EditArgs,
    raw: &str,
    secret: Option<&'static str>,
    credential: Result<Credential>,
) -> Result<()> {
    if let Some(kind) = secret {
        return Err(guard::refusal(kind));
    }
    let job = Job::prepare(services, raw.trim(), credential?)?;
    file::write(&args.file, &file::replacement(raw, &job.run()?))
}
