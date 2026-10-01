use super::*;
use crate::domain::release::ReleaseVersion;

const PAGE: &str = "https://github.com/Keynodex/cleanping/releases/tag/v0.5.0";
const UNCHANGED: &str = "Nothing was changed: this command only checks for a newer version.";

fn v(text: &str) -> ReleaseVersion {
    ReleaseVersion::parse(text).unwrap()
}

fn available() -> UpdateReport {
    UpdateReport::Available {
        running: v("0.4.0"),
        latest: v("0.5.0"),
        page: PAGE.to_string(),
    }
}

#[test]
fn up_to_date_is_one_line() {
    let report = UpdateReport::UpToDate {
        running: v("0.4.0"),
    };
    assert_eq!(
        update_text(&report, "linux", "x86_64"),
        "CleanPing 0.4.0 is the latest version.\n"
    );
}

#[test]
fn on_linux_a_newer_version_points_to_the_page_and_the_readme() {
    let text = update_text(&available(), "linux", "x86_64");
    assert!(
        text.starts_with("CleanPing 0.5.0 is available (you have 0.4.0).\n"),
        "{text}"
    );
    assert!(text.contains(&format!("Release page: {PAGE}\n")), "{text}");
    assert!(text.contains(README_INSTALL), "{text}");
    assert!(text.trim_end().ends_with(UNCHANGED), "{text}");
    assert!(!text.contains("shasum"), "no Mac steps on Linux: {text}");
}

#[test]
fn on_a_mac_the_installer_commands_come_first() {
    let text = update_text(&available(), "macos", "aarch64");
    let download = "  curl -fsSLO https://github.com/Keynodex/cleanping/releases/latest/download/install-cleanping-mac.sh\n";
    let run = "  sh install-cleanping-mac.sh\n";
    let (d, r) = (text.find(download), text.find(run));
    assert!(
        d.is_some() && r.is_some(),
        "installer commands missing in {text}"
    );
    assert!(d < r, "download before run: {text}");
    assert!(
        text.find("shasum -a 256 -c") > r,
        "the manual archive steps are the alternative, after the installer: {text}"
    );
    assert!(
        !text.contains("| sh"),
        "never a download-and-run line: {text}"
    );
    assert!(
        !text.contains("install-cleanping-mac.sh v"),
        "the installer takes no version argument: {text}"
    );
}

#[test]
fn on_a_mac_a_newer_version_shows_the_readme_commands_for_its_archive() {
    let text = update_text(&available(), "macos", "aarch64");
    let name = "cleanping-v0.5.0-aarch64-apple-darwin";
    for command in [
        format!("  shasum -a 256 -c {name}.tar.gz.sha256\n"),
        format!("  tar xzf {name}.tar.gz\n"),
        format!("  install {name}/cleanping ~/.local/bin/\n"),
    ] {
        assert!(text.contains(&command), "missing {command:?} in {text}");
    }
    assert!(text.contains(&format!("Release page: {PAGE}\n")), "{text}");
    assert!(text.trim_end().ends_with(UNCHANGED), "{text}");
    assert!(
        !text.contains("| sh"),
        "never a download-and-run line: {text}"
    );
}

#[test]
fn an_intel_mac_gets_the_intel_archive() {
    let text = update_text(&available(), "macos", "x86_64");
    assert!(
        text.contains("  tar xzf cleanping-v0.5.0-x86_64-apple-darwin.tar.gz\n"),
        "{text}"
    );
    assert!(!text.contains("aarch64"), "{text}");
}

#[test]
fn a_mac_without_a_published_build_gets_the_general_steps() {
    let text = update_text(&available(), "macos", "powerpc");
    assert!(text.contains(README_INSTALL), "{text}");
    assert!(!text.contains("shasum"), "{text}");
}

#[test]
fn a_development_build_is_told_so_without_install_steps() {
    let report = UpdateReport::Ahead {
        running: v("0.5.0"),
        latest: v("0.4.0"),
    };
    for (os, arch) in [("linux", "x86_64"), ("macos", "aarch64")] {
        let text = update_text(&report, os, arch);
        assert_eq!(
            text,
            "CleanPing 0.5.0 is newer than the latest release (0.4.0), so this is a development \
             build. Nothing was changed.\n"
        );
    }
}
