//! `cleanping writer`: a text-only terminal mode. Typed text is never run; Ctrl+G fixes the
//! line in place and Enter copies it. The work is done by `shell/writer.zsh` inside an
//! interactive zsh that reads none of the user's own startup files. It is a guard against
//! accidents for someone new to terminals, not a security sandbox.

mod launch;
mod rc;
mod terminal;

use std::convert::Infallible;
use std::io::IsTerminal;

use cleanping_core::domain::errors::{CleanpingError, Result};
use cleanping_core::infrastructure::paths::data_dir_from;

/// The startup file of the writer's zsh.
const SCRIPT: &str = include_str!("../../shell/writer.zsh");

/// Start the writer. It replaces this process with zsh, so it only returns on an error.
pub fn run() -> Result<Infallible> {
    terminal::require(
        std::io::stdin().is_terminal(),
        std::io::stdout().is_terminal(),
    )?;
    let rc_dir = data_dir_from(|name| std::env::var_os(name))?.join("writer");
    rc::install(&rc_dir).map_err(|_| {
        CleanpingError::Storage("Could not write the writer's startup file.".into())
    })?;
    let bin_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(std::path::Path::to_path_buf))
        .ok_or_else(|| {
            CleanpingError::Storage("Could not find the folder cleanping is installed in.".into())
        })?;
    Err(launch::exec_zsh(&rc_dir, &bin_dir))
}
