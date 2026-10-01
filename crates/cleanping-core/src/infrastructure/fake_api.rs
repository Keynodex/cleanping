//! A fake chat-completions server on 127.0.0.1 for adapter tests: one canned response per
//! connection, and the raw requests it saw. Tests never reach a real provider.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

/// Serves one canned response per connection; records the raw requests it saw.
pub(crate) struct FakeServer {
    pub(crate) url: String,
    pub(crate) requests: Arc<Mutex<Vec<String>>>,
}

pub(crate) fn serve(responses: Vec<String>) -> FakeServer {
    serve_raw(responses.into_iter().map(String::into_bytes).collect())
}

/// Like `serve`, for responses that are not valid UTF-8 (compressed bodies).
pub(crate) fn serve_raw(responses: Vec<Vec<u8>>) -> FakeServer {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!(
        "http://127.0.0.1:{}/v1/chat/completions",
        listener.local_addr().unwrap().port()
    );
    let requests = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&requests);
    std::thread::spawn(move || {
        for response in responses {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let mut buffer = Vec::new();
            let mut chunk = [0u8; 4096];
            loop {
                let n = stream.read(&mut chunk).unwrap_or(0);
                buffer.extend_from_slice(&chunk[..n]);
                let text = String::from_utf8_lossy(&buffer).to_string();
                if let Some(end) = text.find("\r\n\r\n") {
                    let length = text[..end]
                        .lines()
                        .find_map(|l| {
                            l.to_ascii_lowercase()
                                .strip_prefix("content-length:")
                                .map(|v| v.trim().parse::<usize>().unwrap())
                        })
                        .unwrap_or(0);
                    if buffer.len() >= end + 4 + length {
                        break;
                    }
                }
                if n == 0 {
                    break;
                }
            }
            seen.lock()
                .unwrap()
                .push(String::from_utf8_lossy(&buffer).to_string());
            let _ = stream.write_all(&response);
        }
    });
    FakeServer { url, requests }
}

pub(crate) fn ok_json(body: &str) -> String {
    format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
}

pub(crate) fn status(code: u16, reason: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {code} {reason}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}
