//! Test harness: an isolated environment for the real binary plus a fake local API server.
#![allow(dead_code)]

pub mod pty;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use cleanping_core::application::ports::{CredentialRepository, SecretStore};
use cleanping_core::domain::models::CredentialInput;
use cleanping_core::infrastructure::secrets_file::JsonSecretStore;
use cleanping_core::infrastructure::sqlite_db::Database;
use cleanping_core::infrastructure::sqlite_repositories::SqliteCredentialRepository;

pub struct Out {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub struct Sandbox {
    pub dir: tempfile::TempDir,
}

impl Sandbox {
    pub fn new() -> Self {
        Self {
            dir: tempfile::tempdir().unwrap(),
        }
    }

    pub fn db_path(&self) -> std::path::PathBuf {
        self.dir.path().join("data/cleanping/cleanping.db")
    }

    pub fn run(&self, args: &[&str], stdin: Option<&str>) -> Out {
        self.run_with_path(args, stdin, None)
    }

    pub fn run_with_path(&self, args: &[&str], stdin: Option<&str>, path: Option<&str>) -> Out {
        self.run_full(args, stdin, path, &[])
    }

    /// Same as `run`, with extra process environment variables (for example `HTTP_PROXY`).
    pub fn run_with_env(&self, args: &[&str], stdin: Option<&str>, extra: &[(&str, &str)]) -> Out {
        self.run_full(args, stdin, None, extra)
    }

    /// The whole process environment a test run gets: the sandbox dirs, a fixed PATH, `extra`.
    pub fn environment(&self, path: Option<&str>, extra: &[(&str, &str)]) -> Vec<(String, String)> {
        let root = self.dir.path();
        let mut pairs = vec![
            ("HOME".to_string(), root.display().to_string()),
            (
                "XDG_CONFIG_HOME".to_string(),
                root.join("config").display().to_string(),
            ),
            (
                "XDG_DATA_HOME".to_string(),
                root.join("data").display().to_string(),
            ),
            (
                "PATH".to_string(),
                path.unwrap_or("/usr/bin:/bin").to_string(),
            ),
        ];
        pairs.extend(extra.iter().map(|(n, v)| (n.to_string(), v.to_string())));
        pairs
    }

    /// A PATH that finds the freshly built `cleanping` first, as an installed one would be.
    pub fn path_with_binary(&self) -> String {
        let binary_dir = std::path::Path::new(env!("CARGO_BIN_EXE_cleanping"))
            .parent()
            .unwrap()
            .to_path_buf();
        format!("{}:/usr/bin:/bin", binary_dir.display())
    }

    /// The real binary with a scrubbed process environment pointing at the sandbox.
    pub fn command(&self, args: &[&str], path: Option<&str>, extra: &[(&str, &str)]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_cleanping"));
        command
            .args(args)
            .env_clear()
            .envs(self.environment(path, extra));
        command
    }

    /// Run with stdout already closed on the reading side, like `cleanping keys list | head`.
    pub fn run_with_closed_stdout(&self, args: &[&str]) -> Out {
        let mut child = self
            .command(args, None, &[])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        drop(child.stdout.take());
        let output = child.wait_with_output().unwrap();
        Out {
            code: output.status.code().unwrap_or(-1),
            stdout: String::new(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        }
    }

    /// Run with stderr already closed on the reading side, like `cleanping ... 2>&1 | head -0`.
    pub fn run_with_closed_stderr(&self, args: &[&str]) -> Out {
        let mut child = self
            .command(args, None, &[])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        drop(child.stderr.take());
        let output = child.wait_with_output().unwrap();
        Out {
            code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::new(),
        }
    }

    /// Run with stdout connected to `/dev/full`, where every write fails.
    pub fn run_with_full_disk(&self, args: &[&str]) -> Out {
        let full = std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .unwrap();
        let output = self
            .command(args, None, &[])
            .stdin(Stdio::null())
            .stdout(Stdio::from(full))
            .stderr(Stdio::piped())
            .output()
            .unwrap();
        Out {
            code: output.status.code().unwrap_or(-1),
            stdout: String::new(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        }
    }

    fn run_full(
        &self,
        args: &[&str],
        stdin: Option<&str>,
        path: Option<&str>,
        extra: &[(&str, &str)],
    ) -> Out {
        let mut child = self
            .command(args, path, extra)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut input = child.stdin.take().unwrap();
        if let Some(text) = stdin {
            // The CLI may legitimately stop reading early (oversized input), closing the pipe.
            let _ = input.write_all(text.as_bytes());
        }
        drop(input);
        let output = child.wait_with_output().unwrap();
        Out {
            code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        }
    }

    /// Put a credential row straight into the database (no key, no selection), as older
    /// versions could leave it, for example two names that differ only in case.
    pub fn insert_credential(&self, name: &str, url: &str) {
        let db = Database::new(self.db_path());
        db.migrate().unwrap();
        SqliteCredentialRepository::new(db)
            .upsert(&CredentialInput {
                name: name.into(),
                api_url: url.into(),
                model: "test-model".into(),
                api_key: String::new(),
            })
            .unwrap();
    }

    fn secret_store(&self) -> JsonSecretStore {
        JsonSecretStore::new(self.dir.path().join("config/cleanping/secrets.json"))
    }

    /// Put a secret straight into the key file.
    pub fn store_secret(&self, name: &str, secret: &str) {
        self.secret_store().set(name, secret).unwrap();
    }

    /// Whether a key is stored under exactly this name (never returns the key itself).
    pub fn has_secret(&self, name: &str) -> bool {
        self.secret_store().get(name).unwrap().is_some()
    }

    /// Save a keyless credential that points at a local fake server.
    pub fn add_local(&self, name: &str, server: &FakeServer) {
        let out = self.run(
            &[
                "keys",
                "add",
                "--name",
                name,
                "--url",
                &server.url,
                "--model",
                "test-model",
            ],
            None,
        );
        assert_eq!(out.code, 0, "keys add failed: {}", out.stderr);
    }
}

pub struct FakeServer {
    pub url: String,
    pub requests: Arc<Mutex<Vec<String>>>,
}

impl FakeServer {
    fn body(&self, n: usize) -> serde_json::Value {
        let raw = self.requests.lock().unwrap()[n].clone();
        serde_json::from_str(raw.split("\r\n\r\n").nth(1).unwrap()).unwrap()
    }

    /// The user message of the n-th request, as the API would see it.
    pub fn user_text(&self, n: usize) -> String {
        self.body(n)["messages"][1]["content"]
            .as_str()
            .unwrap()
            .to_string()
    }

    pub fn system_text(&self, n: usize) -> String {
        self.body(n)["messages"][0]["content"]
            .as_str()
            .unwrap()
            .to_string()
    }
}

pub fn ok_reply(content: &str) -> String {
    let body = serde_json::json!({"choices": [{"message": {"content": content}}]}).to_string();
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

pub fn status_reply(code: u16, body: &str) -> String {
    format!(
        "HTTP/1.1 {code} X\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

/// One canned response per incoming connection, in order.
pub fn serve(responses: Vec<String>) -> FakeServer {
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
            let _ = stream.write_all(response.as_bytes());
        }
    });
    FakeServer { url, requests }
}
