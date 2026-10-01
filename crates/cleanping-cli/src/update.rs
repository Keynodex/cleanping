//! `cleanping update`: ask GitHub for the newest release and say how to install it. It only
//! checks: nothing is downloaded, installed or changed, and no database or file is opened.

use cleanping_core::application::update_check::UpdateCheck;
use cleanping_core::application::update_text::update_text;
use cleanping_core::domain::errors::Result;
use cleanping_core::infrastructure::github_releases::GithubReleases;

/// The version of this program, as the check compares it and announces it.
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn run() -> Result<()> {
    let report = UpdateCheck::new(GithubReleases::from_env(VERSION)).run(VERSION)?;
    crate::output::write(&update_text(
        &report,
        std::env::consts::OS,
        std::env::consts::ARCH,
    ))
}
