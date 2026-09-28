//! Owner-only files and folders. The one place that knows how to keep personal data private.

use std::fs;
use std::io;
use std::path::Path;

/// Create `directory` (and missing parents) owner-only. A folder that already exists with
/// looser permissions is tightened; if that is impossible the error is returned, so secrets
/// are never written into a folder other people can read (fail closed).
#[cfg(unix)]
pub fn ensure_private_dir(directory: &Path) -> io::Result<()> {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(directory)?;
    if fs::metadata(directory)?.permissions().mode() & 0o077 != 0 {
        fs::set_permissions(directory, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

#[cfg(not(unix))]
pub fn ensure_private_dir(directory: &Path) -> io::Result<()> {
    fs::create_dir_all(directory)
}

/// Best effort: make an existing file owner-only. A missing file is not an error.
#[cfg(unix)]
pub fn tighten_file(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if fs::metadata(path).is_ok_and(|m| m.permissions().mode() & 0o077 != 0) {
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
}

#[cfg(not(unix))]
pub fn tighten_file(_path: &Path) {}

fn private_options() -> fs::OpenOptions {
    let mut options = fs::OpenOptions::new();
    options.write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
}

/// A brand-new owner-only file; fails if the path already exists (never adopts a stale one).
pub fn create_private_file(path: &Path) -> io::Result<fs::File> {
    private_options().create_new(true).open(path)
}

/// An owner-only file that is created if missing and otherwise left as it is (for lock files).
pub fn open_private_file(path: &Path) -> io::Result<fs::File> {
    private_options().create(true).truncate(false).open(path)
}
