//! The file the host app hands us: read it safely and write the accepted text back.

use std::fs::File;
use std::path::Path;

use cleanping_core::domain::errors::{CleanpingError, Result};

use crate::input::read_capped;

fn unreadable() -> CleanpingError {
    CleanpingError::Validation("Could not read the file to edit.".into())
}

/// A regular file of valid UTF-8, no larger than the input limit.
pub fn read(path: &Path) -> Result<String> {
    let file = File::open(path).map_err(|_| unreadable())?;
    if !file.metadata().is_ok_and(|meta| meta.is_file()) {
        return Err(unreadable());
    }
    read_capped(file)
}

/// The new file content: the reply, with the one line ending the file already had.
pub fn replacement(raw: &str, edited: &str) -> String {
    let ending = if raw.ends_with("\r\n") {
        "\r\n"
    } else if raw.ends_with('\n') {
        "\n"
    } else {
        ""
    };
    format!("{}{ending}", edited.trim_end())
}

pub fn write(path: &Path, content: &str) -> Result<()> {
    std::fs::write(path, content).map_err(|_| {
        CleanpingError::Storage("Could not write the edited text back to the file.".into())
    })
}

#[cfg(test)]
#[path = "file_tests.rs"]
mod tests;
