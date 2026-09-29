//! JSON secret store: values live in one 0600 file, never logged or echoed.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

use super::private_fs::{create_private_file, ensure_private_dir, open_private_file, tighten_file};
use crate::application::ports::SecretStore;
use crate::domain::errors::{CleanpingError, Result};

/// [`SecretStore`] kept in one JSON file of name/key pairs. Writes replace the file atomically
/// while holding a lock file; on Unix the file is owner-only (0600) and its folder 0700. A
/// symlinked file is followed. A file that is not a JSON object of strings is a `Storage` error
/// and is left untouched.
pub struct JsonSecretStore {
    path: PathBuf,
}

fn storage(message: impl Into<String>) -> CleanpingError {
    CleanpingError::Storage(message.into())
}

fn cannot_write(error: std::io::Error) -> CleanpingError {
    storage(format!("Cannot write the secret store: {error}."))
}

/// Distinguishes temp files written by threads of one process; the pid separates processes.
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

impl JsonSecretStore {
    /// Use the file at `path` (usually
    /// [`secrets_path`](crate::infrastructure::paths::secrets_path)). Nothing is read or created
    /// until first use.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Where the data really lives: a symlinked secrets file (dotfile managers) is followed
    /// so the link survives a write.
    fn target(&self) -> PathBuf {
        let is_link = fs::symlink_metadata(&self.path).is_ok_and(|m| m.file_type().is_symlink());
        if is_link {
            fs::canonicalize(&self.path).unwrap_or_else(|_| self.path.clone())
        } else {
            self.path.clone()
        }
    }

    /// Run a read-modify-write while holding an exclusive lock, so two `cleanping` processes
    /// cannot overwrite each other's changes.
    fn locked<T>(&self, action: impl FnOnce() -> Result<T>) -> Result<T> {
        let directory = self.path.parent().unwrap_or_else(|| Path::new("."));
        ensure_private_dir(directory).map_err(cannot_write)?;
        let mut lock_name = self.path.file_name().unwrap_or_default().to_os_string();
        lock_name.push(".lock");
        let lock = open_private_file(&self.path.with_file_name(lock_name)).map_err(cannot_write)?;
        lock.lock().map_err(cannot_write)?;
        action() // the lock is released when `lock` is dropped
    }

    fn read(&self) -> Result<BTreeMap<String, String>> {
        let target = self.target();
        let raw = match fs::read_to_string(&target) {
            Ok(raw) => raw,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
            Err(e) => return Err(storage(format!("Cannot read the secret store: {e}."))),
        };
        tighten_file(&target);
        let value: Value = serde_json::from_str(&raw).map_err(|_| {
            storage("Secret store is not valid JSON; fix or remove it before saving keys.")
        })?;
        let Value::Object(object) = value else {
            return Err(storage("Secret store must contain a JSON object."));
        };
        object
            .into_iter()
            .map(|(name, secret)| match secret {
                Value::String(text) => Ok((name, text)),
                _ => Err(storage("Secret store values must be text.")),
            })
            .collect()
    }

    fn write(&self, data: &BTreeMap<String, String>) -> Result<()> {
        let target = self.target();
        let mut name = target.file_name().unwrap_or_default().to_os_string();
        name.push(format!(
            ".{}.{}.tmp",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let tmp = target.with_file_name(name);
        let result = write_private_file(&tmp, data).and_then(|()| fs::rename(&tmp, &target));
        if let Err(e) = result {
            let _ = fs::remove_file(&tmp);
            return Err(cannot_write(e));
        }
        Ok(())
    }
}

/// A brand-new file every time: a stale temp file with looser permissions is never reused.
fn write_private_file(tmp: &Path, data: &BTreeMap<String, String>) -> std::io::Result<()> {
    let mut file = create_private_file(tmp)?;
    file.write_all(serde_json::to_string_pretty(data)?.as_bytes())?;
    file.write_all(b"\n")?;
    file.sync_all()
}

impl SecretStore for JsonSecretStore {
    fn get(&self, name: &str) -> Result<Option<String>> {
        Ok(self.read()?.remove(name))
    }

    fn set(&self, name: &str, secret: &str) -> Result<()> {
        self.locked(|| {
            let mut data = self.read()?;
            data.insert(name.to_string(), secret.to_string());
            self.write(&data)
        })
    }

    fn delete(&self, name: &str) -> Result<()> {
        self.locked(|| {
            let mut data = self.read()?;
            if data.remove(name).is_some() {
                self.write(&data)?;
            }
            Ok(())
        })
    }
}

#[cfg(test)]
#[path = "secrets_file_tests.rs"]
mod tests;
