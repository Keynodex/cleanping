//! Writing to stdout without panics: a closed pipe ends quietly, any other failure is reported.

use std::io::{ErrorKind, Write};

use cleanping_core::domain::errors::{CleanpingError, Result};

/// Write `text` as is. `cleanping keys list | head` closes the pipe early; that is not an error.
pub fn write(text: &str) -> Result<()> {
    let mut out = std::io::stdout().lock();
    match out.write_all(text.as_bytes()).and_then(|()| out.flush()) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::BrokenPipe => Ok(()),
        Err(_) => Err(CleanpingError::Storage(
            "Could not write the output.".into(),
        )),
    }
}

/// Say something on stderr as `cleanping: <message>`. If stderr is gone (closed pipe, full
/// disk) there is nobody to tell, and that must not turn the real outcome into a panic.
pub fn warn(message: &str) {
    let _ = writeln!(std::io::stderr(), "cleanping: {message}");
}

/// Write `text` and a newline.
pub fn line(text: &str) -> Result<()> {
    write(&format!("{text}\n"))
}
