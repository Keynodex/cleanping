//! Checks whether a newer CleanPing release exists. It only asks and compares: nothing is
//! downloaded, installed or changed.

use super::ports::ReleaseSource;
use crate::domain::errors::{CleanpingError, Result};
use crate::domain::release::{update_status, ReleaseVersion, UpdateStatus};
use crate::domain::update_source::release_page;

/// The error when the reply has no usable version (including a pre-release tag).
pub const NOT_UNDERSTOOD: &str =
    "Could not check for updates: the reply from GitHub was not understood.";

/// The outcome of a successful check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateReport {
    /// The running version is the newest release.
    UpToDate {
        /// The running version.
        running: ReleaseVersion,
    },
    /// A newer release exists.
    Available {
        /// The running version.
        running: ReleaseVersion,
        /// The newest release.
        latest: ReleaseVersion,
        /// Its release page, safe to print.
        page: String,
    },
    /// The running version is newer than the newest release (a development build).
    Ahead {
        /// The running version.
        running: ReleaseVersion,
        /// The newest release.
        latest: ReleaseVersion,
    },
}

/// Use case: ask a [`ReleaseSource`] for the newest release and compare it with `running`.
pub struct UpdateCheck<R: ReleaseSource> {
    source: R,
}

impl<R: ReleaseSource> UpdateCheck<R> {
    /// Build the check from the place that knows the newest release.
    pub fn new(source: R) -> Self {
        Self { source }
    }

    /// Compare `running` (this program's version, like `0.4.0`) with the newest release. A
    /// reply whose tag is not a plain `vX.Y.Z` is an `UpdateCheck` error ([`NOT_UNDERSTOOD`]);
    /// so are the source's own failures.
    pub fn run(&self, running: &str) -> Result<UpdateReport> {
        let running = ReleaseVersion::parse(running).ok_or_else(|| {
            CleanpingError::UpdateCheck(
                "Could not check for updates: this build has no plain version number.".into(),
            )
        })?;
        let release = self.source.latest()?;
        let latest = ReleaseVersion::parse(&release.tag_name)
            .ok_or_else(|| CleanpingError::UpdateCheck(NOT_UNDERSTOOD.into()))?;
        Ok(match update_status(running, latest) {
            UpdateStatus::UpToDate => UpdateReport::UpToDate { running },
            UpdateStatus::Ahead => UpdateReport::Ahead { running, latest },
            UpdateStatus::Available => UpdateReport::Available {
                running,
                latest,
                page: release_page(release.html_url.as_deref()),
            },
        })
    }
}

#[cfg(test)]
#[path = "update_check_tests.rs"]
mod tests;
