use super::*;
use std::collections::HashMap;

fn check(pairs: &[(&str, &str)]) -> Option<&'static str> {
    let env: HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect();
    unusable_variable(|name| env.get(name).cloned())
}

#[test]
fn no_proxy_or_empty_ones_are_fine() {
    assert_eq!(check(&[]), None);
    assert_eq!(check(&[("HTTPS_PROXY", ""), ("http_proxy", "   ")]), None);
}

#[test]
fn ordinary_http_and_https_proxies_are_fine() {
    for good in [
        "http://proxy.example:8080",
        "https://proxy.example:8443",
        "http://user:pw@proxy.example:8080",
        "proxy.example:8080",
    ] {
        assert_eq!(check(&[("HTTPS_PROXY", good)]), None, "{good}");
    }
}

#[test]
fn socks_proxies_are_refused_because_they_are_not_built_in() {
    for socks in [
        "socks5://127.0.0.1:1080",
        "socks4://127.0.0.1:1080",
        "socks5h://127.0.0.1:1080",
        "socks://127.0.0.1:1080",
    ] {
        assert_eq!(check(&[("ALL_PROXY", socks)]), Some("ALL_PROXY"), "{socks}");
    }
}

#[test]
fn a_value_that_does_not_parse_is_refused() {
    for bad in ["http://u:p@[bad", "ftp://proxy.example", "::::"] {
        assert_eq!(check(&[("http_proxy", bad)]), Some("http_proxy"), "{bad}");
    }
}

#[test]
fn the_first_bad_variable_is_named_and_never_its_value() {
    let bad = "socks5://user:secret-pw@127.0.0.1:1";
    let name = check(&[("HTTP_PROXY", bad), ("HTTPS_PROXY", bad)]);
    assert_eq!(name, Some("HTTPS_PROXY"));
    assert!(!name.unwrap().contains("secret-pw"));
}
