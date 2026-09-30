use super::*;

/// A lookup over exactly these variables.
fn env(pairs: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
    move |name| {
        pairs
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| value.to_string())
    }
}

const UTF8_XTERM: &[(&str, &str)] = &[("LANG", "en_US.UTF-8"), ("TERM", "xterm-256color")];

#[test]
fn a_utf8_terminal_gets_blocks_in_green() {
    assert_eq!(
        look_for(env(UTF8_XTERM), true),
        Some(Look {
            unicode: true,
            color: true
        })
    );
}

#[test]
fn nothing_is_shown_when_stderr_is_not_a_terminal() {
    assert_eq!(look_for(env(UTF8_XTERM), false), None);
}

#[test]
fn the_switch_turns_it_off() {
    for value in ["off", "OFF", "0", " off "] {
        let lookup = move |name: &str| match name {
            "CLEANPING_PROGRESS" => Some(value.to_string()),
            other => env(UTF8_XTERM)(other),
        };
        assert_eq!(look_for(lookup, true), None, "{value}");
    }
    let on = |name: &str| match name {
        "CLEANPING_PROGRESS" => Some("on".to_string()),
        other => env(UTF8_XTERM)(other),
    };
    assert!(look_for(on, true).is_some());
}

#[test]
fn no_color_keeps_the_bar_but_not_the_green() {
    let look = look_for(
        env(&[("LANG", "C.UTF-8"), ("TERM", "xterm"), ("NO_COLOR", "1")]),
        true,
    );
    assert_eq!(
        look,
        Some(Look {
            unicode: true,
            color: false
        })
    );
    let empty = look_for(
        env(&[("LANG", "C.UTF-8"), ("TERM", "xterm"), ("NO_COLOR", "")]),
        true,
    );
    assert_eq!(
        empty.map(|l| l.color),
        Some(true),
        "an empty NO_COLOR is unset"
    );
}

#[test]
fn a_dumb_terminal_gets_plain_ascii() {
    let look = look_for(env(&[("LANG", "en_US.UTF-8"), ("TERM", "dumb")]), true);
    assert_eq!(
        look,
        Some(Look {
            unicode: false,
            color: false
        })
    );
}

#[test]
fn a_locale_that_is_not_utf8_gets_ascii() {
    for pairs in [
        &[("LANG", "C"), ("TERM", "xterm")][..],
        &[("TERM", "xterm")][..],
        &[("LANG", "en_US.ISO-8859-1"), ("TERM", "xterm")][..],
    ] {
        let lookup = move |name: &str| {
            pairs
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| value.to_string())
        };
        assert_eq!(look_for(lookup, true).map(|l| l.unicode), Some(false));
    }
}

#[test]
fn lc_all_wins_over_lc_ctype_which_wins_over_lang() {
    let all = env(&[
        ("LC_ALL", "C"),
        ("LC_CTYPE", "en_US.UTF-8"),
        ("TERM", "xterm"),
    ]);
    assert_eq!(look_for(all, true).map(|l| l.unicode), Some(false));
    let ctype = env(&[("LC_CTYPE", "en_US.utf8"), ("LANG", "C"), ("TERM", "xterm")]);
    assert_eq!(look_for(ctype, true).map(|l| l.unicode), Some(true));
    let empty_all = env(&[("LC_ALL", ""), ("LANG", "en_US.UTF-8"), ("TERM", "xterm")]);
    assert_eq!(look_for(empty_all, true).map(|l| l.unicode), Some(true));
}
