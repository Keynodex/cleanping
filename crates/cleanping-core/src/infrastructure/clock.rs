//! UTC timestamps in one format everywhere: 2026-09-28T18:30:00+00:00.

use time::{Duration, OffsetDateTime, UtcOffset};

/// `moment` as a UTC stamp in whole seconds, such as `2026-09-28T18:30:00+00:00`. Every stored
/// time uses this form, so stamps compare correctly as text.
pub fn format_utc(moment: OffsetDateTime) -> String {
    let m = moment.to_offset(UtcOffset::UTC);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}+00:00",
        m.year(),
        u8::from(m.month()),
        m.day(),
        m.hour(),
        m.minute(),
        m.second()
    )
}

/// The current time as a [`format_utc`] stamp.
pub fn utc_now() -> String {
    format_utc(OffsetDateTime::now_utc())
}

/// The stamp for `days` days ago (the start of time if that is out of range).
pub fn days_ago(days: u32) -> String {
    let moment = OffsetDateTime::now_utc()
        .checked_sub(Duration::days(i64::from(days)))
        .unwrap_or(OffsetDateTime::UNIX_EPOCH);
    format_utc(moment)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_like_the_python_release() {
        let moment = OffsetDateTime::from_unix_timestamp(1790620200).unwrap();
        assert_eq!(format_utc(moment), "2026-09-28T18:30:00+00:00");
    }

    #[test]
    fn now_has_the_same_shape() {
        let now = utc_now();
        assert_eq!((now.len(), &now[10..11], &now[19..]), (25, "T", "+00:00"));
    }

    #[test]
    fn days_ago_is_earlier_than_now_and_zero_is_now() {
        assert!(days_ago(1) < utc_now());
        assert!(days_ago(3650) < days_ago(1));
        assert!(days_ago(u32::MAX) <= days_ago(3650));
        assert!(days_ago(0) <= utc_now());
    }
}
