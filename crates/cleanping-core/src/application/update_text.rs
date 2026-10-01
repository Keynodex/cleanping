//! The words `cleanping update` prints for a check's outcome, for a given system. Pure: the
//! caller passes `std::env::consts::OS` and `ARCH`, so every system's text can be tested
//! anywhere.

use super::update_check::UpdateReport;

/// The README's install section, for systems without their own steps here.
pub const README_INSTALL: &str = "https://github.com/Keynodex/cleanping#install";

/// What to print for `report` on a system whose `std::env::consts::OS` is `os` and whose
/// `ARCH` is `arch`. Every value in it was validated by the check, so it holds no control
/// characters.
pub fn update_text(report: &UpdateReport, os: &str, arch: &str) -> String {
    match report {
        UpdateReport::UpToDate { running } => {
            format!("CleanPing {running} is the latest version.\n")
        }
        UpdateReport::Ahead { running, latest } => format!(
            "CleanPing {running} is newer than the latest release ({latest}), so this is a \
             development build. Nothing was changed.\n"
        ),
        UpdateReport::Available {
            running,
            latest,
            page,
        } => {
            let steps = match mac_target(os, arch) {
                Some(target) => mac_steps(&format!("cleanping-v{latest}-{target}")),
                None => format!(
                    "To install it, download the archive for your system from the release page \
                     and follow the install steps in the README: {README_INSTALL}\n"
                ),
            };
            format!(
                "CleanPing {latest} is available (you have {running}).\nRelease page: {page}\n\n\
                 {steps}\n{UNCHANGED}\n"
            )
        }
    }
}

const UNCHANGED: &str = "Nothing was changed: this command only checks for a newer version.";

/// The release build for this Mac, when one is published.
fn mac_target(os: &str, arch: &str) -> Option<&'static str> {
    match (os, arch) {
        ("macos", "aarch64") => Some("aarch64-apple-darwin"),
        ("macos", "x86_64") => Some("x86_64-apple-darwin"),
        _ => None,
    }
}

/// The README's prebuilt-binary steps, with the Mac checksum tool, for the archive `name`.
fn mac_steps(name: &str) -> String {
    format!(
        "To install it on this Mac, download {name}.tar.gz and {name}.tar.gz.sha256 from the \
         release page into one folder, then run these commands in that folder:\n\n  \
         shasum -a 256 -c {name}.tar.gz.sha256\n  \
         tar xzf {name}.tar.gz\n  \
         install {name}/cleanping ~/.local/bin/\n"
    )
}

#[cfg(test)]
#[path = "update_text_tests.rs"]
mod tests;
