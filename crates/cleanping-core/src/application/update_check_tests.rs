use super::*;
use crate::application::ports::LatestRelease;
use std::cell::Cell;

/// Answers with a fixed release, or a fixed failure message; counts how often it was asked.
struct FakeSource {
    reply: std::result::Result<LatestRelease, String>,
    asked: Cell<usize>,
}

impl ReleaseSource for &FakeSource {
    fn latest(&self) -> Result<LatestRelease> {
        self.asked.set(self.asked.get() + 1);
        self.reply.clone().map_err(CleanpingError::UpdateCheck)
    }
}

fn source(tag: &str, page: Option<&str>) -> FakeSource {
    FakeSource {
        reply: Ok(LatestRelease {
            tag_name: tag.to_string(),
            html_url: page.map(str::to_string),
        }),
        asked: Cell::new(0),
    }
}

fn v(text: &str) -> ReleaseVersion {
    ReleaseVersion::parse(text).unwrap()
}

const PAGE: &str = "https://github.com/Keynodex/cleanping/releases/tag/v0.5.0";

#[test]
fn the_same_version_is_up_to_date() {
    let fake = source("v0.4.0", Some(PAGE));
    let report = UpdateCheck::new(&fake).run("0.4.0").unwrap();
    assert_eq!(
        report,
        UpdateReport::UpToDate {
            running: v("0.4.0")
        }
    );
    assert_eq!(fake.asked.get(), 1);
}

#[test]
fn a_newer_release_is_available_with_its_page() {
    let fake = source("v0.5.0", Some(PAGE));
    let report = UpdateCheck::new(&fake).run("0.4.0").unwrap();
    assert_eq!(
        report,
        UpdateReport::Available {
            running: v("0.4.0"),
            latest: v("0.5.0"),
            page: PAGE.to_string(),
        }
    );
}

#[test]
fn an_untrusted_page_is_replaced_by_the_standard_one() {
    let fake = source("v0.10.0", Some("https://evil.example/cleanping"));
    let report = UpdateCheck::new(&fake).run("0.9.0").unwrap();
    let UpdateReport::Available { page, .. } = report else {
        panic!("0.10.0 is newer than 0.9.0: {report:?}");
    };
    assert_eq!(
        page,
        "https://github.com/Keynodex/cleanping/releases/latest"
    );
}

#[test]
fn a_build_newer_than_the_latest_release_is_ahead() {
    let fake = source("v0.4.0", None);
    let report = UpdateCheck::new(&fake).run("0.5.0").unwrap();
    assert_eq!(
        report,
        UpdateReport::Ahead {
            running: v("0.5.0"),
            latest: v("0.4.0"),
        }
    );
}

#[test]
fn a_tag_that_is_not_a_plain_version_is_not_understood() {
    for tag in ["v0.5.0-rc.1", "latest", "", "v0.5.0\u{1b}[2J", "v0.5"] {
        let fake = source(tag, Some(PAGE));
        let error = UpdateCheck::new(&fake).run("0.4.0").unwrap_err();
        assert_eq!(
            error,
            CleanpingError::UpdateCheck(NOT_UNDERSTOOD.into()),
            "{tag:?}"
        );
    }
}

#[test]
fn a_running_version_that_is_not_plain_is_not_compared() {
    let fake = source("v0.5.0", Some(PAGE));
    let error = UpdateCheck::new(&fake).run("0.5.0-dev").unwrap_err();
    assert!(matches!(error, CleanpingError::UpdateCheck(_)), "{error:?}");
    assert_eq!(
        fake.asked.get(),
        0,
        "nothing is asked when it cannot be compared"
    );
}

#[test]
fn the_source_failure_is_passed_on_unchanged() {
    let message = "Could not check for updates: offline.";
    let fake = FakeSource {
        reply: Err(message.into()),
        asked: Cell::new(0),
    };
    assert_eq!(
        UpdateCheck::new(&fake).run("0.4.0").unwrap_err(),
        CleanpingError::UpdateCheck(message.into())
    );
}
