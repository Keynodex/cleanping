use super::*;

fn at(input_bytes: usize, millis: u64) -> u8 {
    estimate_progress(input_bytes, Duration::from_millis(millis)).percent
}

/// Every 50 ms over ten minutes.
fn steps() -> impl Iterator<Item = u64> {
    (0..=12_000).map(|n| n * 50)
}

#[test]
fn nothing_has_happened_at_the_start() {
    assert_eq!(at(0, 0), 0);
    assert_eq!(at(500, 0), 0);
    assert_eq!(at(usize::MAX, 0), 0);
}

#[test]
fn it_never_goes_backwards() {
    for size in [0, 40, 2_000, 200_000] {
        let mut last = 0;
        for millis in steps() {
            let now = at(size, millis);
            assert!(now >= last, "went back from {last} to {now} at {millis} ms");
            last = now;
        }
    }
}

#[test]
fn it_is_held_at_95_and_never_says_100() {
    for size in [0, 40, 2_000, 200_000] {
        assert!(steps().all(|millis| at(size, millis) <= HELD_AT));
    }
    let long_after = Duration::from_secs(3600);
    assert_eq!(estimate_progress(40, long_after).percent, 95);
    assert_eq!(estimate_progress(200_000, long_after).percent, 95);
    assert_eq!(estimate_progress(40, Duration::MAX).percent, 95);
}

#[test]
fn it_moves_from_the_first_second() {
    assert!(at(40, 1_000) > 0, "{}", at(40, 1_000));
    assert!(at(40, 1_500) < at(40, 3_000));
}

#[test]
fn it_is_most_of_the_way_at_the_expected_time_and_full_at_twice_that() {
    let expected = expected_duration(2_000);
    let there = estimate_progress(2_000, expected).percent;
    assert!((80..=86).contains(&there), "{there}");
    assert_eq!(estimate_progress(2_000, expected * 2).percent, HELD_AT);
}

#[test]
fn a_bigger_text_is_never_further_along_after_the_same_wait() {
    for millis in steps() {
        assert!(at(5_000, millis) <= at(100, millis), "at {millis} ms");
    }
    assert!(at(5_000, 5_000) < at(100, 5_000));
}

#[test]
fn the_expected_time_grows_with_the_text() {
    assert_eq!(expected_duration(0), timing::BASE);
    assert_eq!(
        expected_duration(1_000),
        timing::BASE + timing::PER_KILOBYTE
    );
    assert!(expected_duration(200) < expected_duration(2_000));
}

#[test]
fn a_huge_text_is_expected_to_take_at_most_the_timeout() {
    assert_eq!(expected_duration(usize::MAX), timing::LONGEST);
    assert_eq!(expected_duration(200_000), timing::LONGEST);
    assert!(at(usize::MAX, 60_000) < HELD_AT);
    assert!(at(usize::MAX, 1_000) <= at(0, 1_000));
}
