use super::*;

#[test]
fn without_an_override_the_github_api_is_asked() {
    assert_eq!(update_api_url(None), LATEST_RELEASE_API);
    assert_eq!(
        LATEST_RELEASE_API,
        "https://api.github.com/repos/Keynodex/cleanping/releases/latest"
    );
}

#[test]
fn a_plain_http_loopback_override_is_used() {
    for url in [
        "http://127.0.0.1:8080/releases/latest",
        "http://localhost:8080/x",
        "http://[::1]:8080/x",
    ] {
        assert_eq!(update_api_url(Some(url)), url, "{url}");
    }
}

#[test]
fn an_override_that_could_leave_this_machine_is_ignored() {
    for url in [
        "https://example.com/releases/latest",
        "http://example.com:8080/x",
        "https://127.0.0.1:8080/x",
        "http://127.0.0.1.example.com:8080/x",
        "http://localhost.example.com:8080/x",
        "http://user@127.0.0.1:8080/x",
        "http://user:pw@localhost:8080/x",
        "http://127.0.0.2:8080/x",
        "http://0.0.0.0:8080/x",
        "http://10.0.0.1:8080/x",
        "ftp://127.0.0.1/x",
        "file:///etc/passwd",
        "127.0.0.1:8080",
        "",
        "not a url",
    ] {
        assert_eq!(
            update_api_url(Some(url)),
            LATEST_RELEASE_API,
            "{url:?} must be ignored"
        );
    }
}

#[test]
fn a_release_page_on_the_cleanping_repository_is_kept() {
    let page = "https://github.com/Keynodex/cleanping/releases/tag/v0.5.0";
    assert_eq!(release_page(Some(page)), page);
}

#[test]
fn any_other_release_page_is_replaced_by_the_standard_one() {
    let long = format!("{RELEASE_PAGE_PREFIX}{}", "a".repeat(300));
    for page in [
        "https://evil.example/Keynodex/cleanping/releases/tag/v0.5.0",
        "https://github.com/Keynodex/cleanping-evil/releases",
        "https://github.com/Keynodex/cleanping",
        "http://github.com/Keynodex/cleanping/releases/tag/v0.5.0",
        "https://github.com.evil.example/Keynodex/cleanping/x",
        "https://github.com/Keynodex/cleanping/releases\u{1b}[2J",
        "https://github.com/Keynodex/cleanping/releases\nrun this",
        "https://github.com/Keynodex/cleanping/releases tag",
        "https://github.com/Keynodex/cleanping/r\u{202e}eleases",
        long.as_str(),
        "",
    ] {
        assert_eq!(release_page(Some(page)), RELEASES_PAGE, "{page:?}");
    }
    assert_eq!(release_page(None), RELEASES_PAGE);
}
