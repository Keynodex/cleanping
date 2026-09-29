//! A real pseudo-terminal session (Linux `script`), driven by what it prints, not by sleeps.
#![allow(dead_code)]

use std::io::{Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct Pty {
    child: Child,
    stdin: Option<ChildStdin>,
    screen: Arc<Mutex<Vec<u8>>>,
}

impl Pty {
    /// Start `shell_command` inside a pty with exactly the environment `env` (see
    /// `Sandbox::environment`). Everything the program prints appears on `screen()`.
    pub fn start(env: &[(String, String)], shell_command: &str) -> Self {
        let mut child = Command::new("script")
            .args(["-qec", shell_command, "/dev/null"])
            .env_clear()
            .envs(env.iter().map(|(name, value)| (name, value)))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("the `script` tool is needed for terminal tests");
        let stdin = child.stdin.take();
        let mut stdout = child.stdout.take().unwrap();
        let screen = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&screen);
        std::thread::spawn(move || {
            let mut chunk = [0u8; 4096];
            while let Ok(n) = stdout.read(&mut chunk) {
                if n == 0 {
                    break;
                }
                sink.lock().unwrap().extend_from_slice(&chunk[..n]);
            }
        });
        Self {
            child,
            stdin,
            screen,
        }
    }

    pub fn send(&mut self, keys: &str) {
        let input = self.stdin.as_mut().expect("stdin already closed");
        input.write_all(keys.as_bytes()).unwrap();
        input.flush().unwrap();
    }

    pub fn screen(&self) -> String {
        String::from_utf8_lossy(&self.screen.lock().unwrap()).to_string()
    }

    /// What the program printed, with colors and cursor movements removed.
    pub fn plain_screen(&self) -> String {
        strip_escapes(&self.screen())
    }

    pub fn wait_for_text(&self, needle: &str, timeout: Duration) -> bool {
        self.wait_until(timeout, || self.plain_screen().contains(needle))
    }

    /// Poll until `condition` holds; false on timeout.
    pub fn wait_until(&self, timeout: Duration, condition: impl Fn() -> bool) -> bool {
        let started = Instant::now();
        while started.elapsed() < timeout {
            if condition() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        condition()
    }

    pub fn wait_for_screen(&self, needle: &str, timeout: Duration) -> bool {
        self.wait_until(timeout, || self.screen().contains(needle))
    }

    /// Close the terminal's input and wait for the session to end; returns the exit code.
    pub fn finish(mut self, timeout: Duration) -> i32 {
        drop(self.stdin.take());
        let started = Instant::now();
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                return status.code().unwrap_or(-1);
            }
            if started.elapsed() > timeout {
                let _ = self.child.kill();
                panic!("the terminal session did not end in time");
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

/// Drop terminal control sequences (CSI, OSC, charset selection), keeping the visible text.
fn strip_escapes(raw: &str) -> String {
    let mut out = String::new();
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('[') => {
                for next in chars.by_ref() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
            }
            Some(']') => {
                while let Some(next) = chars.next() {
                    if next == '\u{7}' || (next == '\u{1b}' && chars.next_if_eq(&'\\').is_some()) {
                        break;
                    }
                }
            }
            Some('(' | ')') => {
                chars.next();
            }
            _ => {}
        }
    }
    out
}
