//! Starting zsh: the PATH the writer's shell gets and the errors for a zsh that will not start.

use std::ffi::{OsStr, OsString};
use std::io::ErrorKind;
use std::path::Path;

use cleanping_core::domain::errors::CleanpingError;

/// `existing` with `dir` put first, so the startup file can call `cleanping`. `None` when the
/// result cannot be written as a PATH (a folder name containing `:`).
pub fn path_with_first(dir: &Path, existing: Option<&OsStr>) -> Option<OsString> {
    let rest = existing
        .into_iter()
        .flat_map(|value| std::env::split_paths(value));
    std::env::join_paths(std::iter::once(dir.to_path_buf()).chain(rest)).ok()
}

/// What to tell the user when zsh could not be started.
pub fn start_error(error: &std::io::Error) -> CleanpingError {
    CleanpingError::Storage(if error.kind() == ErrorKind::NotFound {
        "cleanping writer needs zsh, and zsh was not found. macOS and most Linux systems \
         have it: install it with your system's package manager."
            .into()
    } else {
        "Could not start zsh for the writer.".to_string()
    })
}

/// Replace this process with an interactive zsh that reads only the startup file in `rc_dir`
/// (`-d` skips the system-wide ones; `-f` would skip ours too) and finds `cleanping` on its
/// PATH. Returns only when zsh could not be started.
#[cfg(unix)]
pub fn exec_zsh(rc_dir: &Path, bin_dir: &Path) -> CleanpingError {
    use std::os::unix::process::CommandExt;
    let Some(path) = path_with_first(bin_dir, std::env::var_os("PATH").as_deref()) else {
        return CleanpingError::Storage(
            "Could not put cleanping on the writer's PATH: its folder name has a colon.".into(),
        );
    };
    let mut zsh = std::process::Command::new("zsh");
    zsh.args(["-d", "-i"]);
    zsh.envs([("ZDOTDIR", rc_dir.as_os_str()), ("PATH", path.as_os_str())]);
    start_error(&zsh.exec())
}

#[cfg(not(unix))]
pub fn exec_zsh(_rc_dir: &Path, _bin_dir: &Path) -> CleanpingError {
    CleanpingError::Storage("cleanping writer needs zsh, which this system does not have.".into())
}

#[cfg(test)]
#[path = "launch_tests.rs"]
mod tests;
