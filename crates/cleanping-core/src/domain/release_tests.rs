use super::*;

fn v(text: &str) -> ReleaseVersion {
    ReleaseVersion::parse(text).unwrap_or_else(|| panic!("version {text:?} must parse"))
}

#[test]
fn plain_and_v_prefixed_versions_parse_to_the_same_value() {
    assert_eq!(v("0.4.0"), v("v0.4.0"));
    assert_eq!(v("v12.0.7").to_string(), "12.0.7");
}

#[test]
fn leading_zeros_are_read_as_numbers() {
    assert_eq!(v("v0.04.00"), v("0.4.0"));
    assert_eq!(v("007.1.2").to_string(), "7.1.2");
}

#[test]
fn the_largest_number_parses_and_one_more_is_refused() {
    assert_eq!(
        v("18446744073709551615.0.0").to_string(),
        "18446744073709551615.0.0"
    );
    assert_eq!(ReleaseVersion::parse("18446744073709551616.0.0"), None);
    assert_eq!(
        ReleaseVersion::parse("1.99999999999999999999999999.0"),
        None
    );
}

#[test]
fn anything_that_is_not_three_plain_numbers_is_refused() {
    for text in [
        "",
        "v",
        "0.4",
        "0.4.0.1",
        "1.2.3-rc.1",
        "v1.2.3-beta",
        "1.2.3+build",
        "V1.2.3",
        "vv1.2.3",
        " 1.2.3",
        "1.2.3\n",
        "1..3",
        ".1.2",
        "+1.2.3",
        "1.+2.3",
        "-1.2.3",
        "1.2.x",
        "\u{661}.2.3", // an Arabic-Indic digit is a digit, but not ASCII
        "1.2.3\u{1b}[2J",
        "latest",
    ] {
        assert_eq!(
            ReleaseVersion::parse(text),
            None,
            "{text:?} must be refused"
        );
    }
}

#[test]
fn update_status_compares_numbers_not_text() {
    let cases = [
        ("0.4.0", "0.5.0", UpdateStatus::Available),
        ("0.9.0", "0.10.0", UpdateStatus::Available),
        ("0.4.0", "1.0.0", UpdateStatus::Available),
        ("0.4.0", "0.4.1", UpdateStatus::Available),
        ("0.4.0", "0.4.0", UpdateStatus::UpToDate),
        ("0.4.0", "v0.04.0", UpdateStatus::UpToDate),
        ("0.10.0", "0.9.0", UpdateStatus::Ahead),
        ("0.5.0", "0.4.0", UpdateStatus::Ahead),
        ("1.0.0", "0.99.99", UpdateStatus::Ahead),
    ];
    for (running, latest, expected) in cases {
        assert_eq!(
            update_status(v(running), v(latest)),
            expected,
            "running {running}, latest {latest}"
        );
    }
}
