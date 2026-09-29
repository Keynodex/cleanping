use super::*;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

struct Fake {
    api_url: String,
    requests: Arc<Mutex<Vec<String>>>,
}

/// Answers the first connection with `response` and records the request line it saw.
fn serve_once(response: Vec<u8>) -> Fake {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let api_url = format!(
        "http://127.0.0.1:{}/v1/chat/completions",
        listener.local_addr().unwrap().port()
    );
    let requests = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&requests);
    std::thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 1024];
        while !buffer.windows(4).any(|w| w == b"\r\n\r\n") {
            let n = stream.read(&mut chunk).unwrap_or(0);
            if n == 0 {
                break;
            }
            buffer.extend_from_slice(&chunk[..n]);
        }
        let request = String::from_utf8_lossy(&buffer).to_string();
        seen.lock()
            .unwrap()
            .push(request.lines().next().unwrap_or("").to_string());
        let _ = stream.write_all(&response);
    });
    Fake { api_url, requests }
}

fn reply(status: &str, body: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .into_bytes()
}

/// An address where nothing listens.
fn closed_url() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    format!("http://127.0.0.1:{port}/v1/chat/completions")
}

fn path_with_program(executable: bool) -> (tempfile::TempDir, std::ffi::OsString) {
    let dir = tempfile::tempdir().unwrap();
    let program = dir.path().join("ollama");
    std::fs::write(&program, "#!/bin/sh\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = if executable { 0o755 } else { 0o644 };
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(mode)).unwrap();
    }
    let path = dir.path().as_os_str().to_owned();
    (dir, path)
}

#[test]
fn an_ollama_answer_lists_its_models() {
    let body = r#"{"models":[{"name":"qwen2.5:7b"},{"name":"llama3:latest"}]}"#;
    let fake = serve_once(reply("200 OK", body));
    let server = probe_with(&fake.api_url, None).unwrap();
    assert_eq!(
        server,
        LocalServer::Running {
            models: vec!["qwen2.5:7b".into(), "llama3:latest".into()]
        }
    );
    assert_eq!(fake.requests.lock().unwrap()[0], "GET /api/tags HTTP/1.1");
}

#[test]
fn an_answer_that_is_not_ollama_is_other() {
    for (status, body) in [
        ("200 OK", "<html>hello</html>"),
        ("200 OK", r#"{"models":"none"}"#),
        ("200 OK", r#"{"data":[]}"#),
        ("404 Not Found", "nope"),
    ] {
        let fake = serve_once(reply(status, body));
        assert_eq!(
            probe_with(&fake.api_url, None),
            Some(LocalServer::Other),
            "{status} {body}"
        );
    }
}

#[test]
fn a_redirect_is_not_followed() {
    let redirect = b"HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:1/x\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec();
    let fake = serve_once(redirect);
    assert_eq!(probe_with(&fake.api_url, None), Some(LocalServer::Other));
}

#[test]
fn a_huge_answer_is_refused_not_read() {
    let body = "a".repeat(1_100_000);
    let fake = serve_once(reply("200 OK", &body));
    assert_eq!(probe_with(&fake.api_url, None), Some(LocalServer::Other));
}

#[test]
fn no_answer_and_no_program_means_not_installed() {
    let empty = tempfile::tempdir().unwrap();
    let path = empty.path().as_os_str().to_owned();
    assert_eq!(
        probe_with(&closed_url(), Some(&path)),
        Some(LocalServer::NotInstalled)
    );
    assert_eq!(
        probe_with(&closed_url(), None),
        Some(LocalServer::NotInstalled)
    );
}

#[test]
fn no_answer_but_the_program_present_means_not_running() {
    let (_dir, path) = path_with_program(true);
    assert_eq!(
        probe_with(&closed_url(), Some(&path)),
        Some(LocalServer::NotRunning)
    );
}

#[cfg(unix)]
#[test]
fn a_file_that_cannot_be_run_is_not_the_program() {
    let (_dir, path) = path_with_program(false);
    assert_eq!(
        probe_with(&closed_url(), Some(&path)),
        Some(LocalServer::NotInstalled)
    );
}

#[test]
fn an_address_that_is_not_local_is_never_asked() {
    assert_eq!(
        probe_with("https://api.example.com/v1/chat/completions", None),
        None
    );
    assert_eq!(probe_with("not a url", None), None);
}
