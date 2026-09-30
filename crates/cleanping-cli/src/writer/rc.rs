//! The writer's private startup file: written fresh on every run, readable only by its owner.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use cleanping_core::infrastructure::private_fs::{create_private_file, ensure_private_dir};

use super::SCRIPT;

/// Put the embedded `writer.zsh` into `dir` as `.zshrc` (the folder is created owner-only).
/// The file is written beside its final place and renamed over it, so two terminals starting
/// at once never see half a file, and a link left at that place is replaced, not followed.
pub fn install(dir: &Path) -> io::Result<()> {
    ensure_private_dir(dir)?;
    let temporary = dir.join(format!(".zshrc.{}.tmp", std::process::id()));
    let _ = fs::remove_file(&temporary);
    let written = create_private_file(&temporary).and_then(|mut file| {
        file.write_all(SCRIPT.as_bytes())?;
        file.sync_all()
    });
    let done = written.and_then(|()| fs::rename(&temporary, dir.join(".zshrc")));
    if done.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    done
}

#[cfg(test)]
#[path = "rc_tests.rs"]
mod tests;
