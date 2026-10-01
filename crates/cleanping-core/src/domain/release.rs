//! Release versions (`v0.4.0`) and how the running version compares with the latest release.

use std::fmt;

/// A release version: three plain numbers. Pre-release and build suffixes are not accepted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReleaseVersion {
    major: u64,
    minor: u64,
    patch: u64,
}

impl ReleaseVersion {
    /// Parse `1.2.3` or `v1.2.3` (each part ASCII digits only, up to `u64::MAX`). Anything else,
    /// including `1.2.3-rc.1`, spaces or a number too large, is `None`.
    pub fn parse(text: &str) -> Option<Self> {
        let digits = text.strip_prefix('v').unwrap_or(text);
        let mut parts = digits.split('.').map(number);
        let version = Self {
            major: parts.next()??,
            minor: parts.next()??,
            patch: parts.next()??,
        };
        parts.next().is_none().then_some(version)
    }
}

/// One part of a version: ASCII digits only (`u64::from_str` would also take a leading `+`).
fn number(part: &str) -> Option<u64> {
    if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    part.parse().ok()
}

impl fmt::Display for ReleaseVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// How the running version stands against the latest release.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpdateStatus {
    /// The running version is the latest release.
    UpToDate,
    /// A newer release exists.
    Available,
    /// The running version is newer than the latest release (a development build).
    Ahead,
}

/// Compare the running version with the latest release.
pub fn update_status(running: ReleaseVersion, latest: ReleaseVersion) -> UpdateStatus {
    match running.cmp(&latest) {
        std::cmp::Ordering::Less => UpdateStatus::Available,
        std::cmp::Ordering::Equal => UpdateStatus::UpToDate,
        std::cmp::Ordering::Greater => UpdateStatus::Ahead,
    }
}

#[cfg(test)]
#[path = "release_tests.rs"]
mod tests;
