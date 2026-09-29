//! XDG locations for runtime data (never inside the repo). Same paths as the Python release.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::domain::errors::{CleanpingError, Result};

/// `$VAR` when it is set to an absolute path, otherwise `$HOME/<fallback>` (`HOME` must be
/// absolute too).
fn xdg_base(
    get: &impl Fn(&str) -> Option<OsString>,
    var: &str,
    fallback: &[&str],
) -> Result<PathBuf> {
    if let Some(dir) = get(var).filter(|v| Path::new(v).is_absolute()) {
        return Ok(PathBuf::from(dir));
    }
    let home = get("HOME")
        .filter(|h| Path::new(h).is_absolute())
        .ok_or_else(|| CleanpingError::Storage("Cannot find your home directory.".into()))?;
    Ok(fallback
        .iter()
        .fold(PathBuf::from(home), |path, part| path.join(part)))
}

/// `$XDG_CONFIG_HOME/cleanping`, or `$HOME/.config/cleanping` when that variable is unset or
/// not an absolute path. `get` looks up an environment variable by name. A `Storage` error when
/// `HOME` is needed but missing or relative.
pub fn config_dir_from(get: impl Fn(&str) -> Option<OsString>) -> Result<PathBuf> {
    Ok(xdg_base(&get, "XDG_CONFIG_HOME", &[".config"])?.join("cleanping"))
}

/// `$XDG_DATA_HOME/cleanping`, or `$HOME/.local/share/cleanping`; the same rules as
/// [`config_dir_from`].
pub fn data_dir_from(get: impl Fn(&str) -> Option<OsString>) -> Result<PathBuf> {
    Ok(xdg_base(&get, "XDG_DATA_HOME", &[".local", "share"])?.join("cleanping"))
}

fn real_env(name: &str) -> Option<OsString> {
    std::env::var_os(name)
}

/// The secret store file: `secrets.json` in the config folder, found from the process
/// environment.
pub fn secrets_path() -> Result<PathBuf> {
    Ok(config_dir_from(real_env)?.join("secrets.json"))
}

/// The database file: `cleanping.db` in the data folder, found from the process environment.
pub fn database_path() -> Result<PathBuf> {
    Ok(data_dir_from(real_env)?.join("cleanping.db"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| ((*k).into(), (*v).into()))
            .collect();
        move |name| map.get(name).map(OsString::from)
    }

    #[test]
    fn xdg_variables_win() {
        let get = env(&[
            ("XDG_CONFIG_HOME", "/x/c"),
            ("XDG_DATA_HOME", "/x/d"),
            ("HOME", "/h"),
        ]);
        assert_eq!(
            config_dir_from(&get).unwrap(),
            PathBuf::from("/x/c/cleanping")
        );
        assert_eq!(
            data_dir_from(&get).unwrap(),
            PathBuf::from("/x/d/cleanping")
        );
    }

    #[test]
    fn empty_or_missing_xdg_falls_back_to_home() {
        let get = env(&[("XDG_CONFIG_HOME", ""), ("HOME", "/h")]);
        assert_eq!(
            config_dir_from(&get).unwrap(),
            PathBuf::from("/h/.config/cleanping")
        );
        assert_eq!(
            data_dir_from(&get).unwrap(),
            PathBuf::from("/h/.local/share/cleanping")
        );
    }

    #[test]
    fn relative_xdg_values_are_ignored_as_the_spec_requires() {
        let get = env(&[
            ("XDG_CONFIG_HOME", "relcfg"),
            ("XDG_DATA_HOME", "./d"),
            ("HOME", "/h"),
        ]);
        assert_eq!(
            config_dir_from(&get).unwrap(),
            PathBuf::from("/h/.config/cleanping")
        );
        assert_eq!(
            data_dir_from(&get).unwrap(),
            PathBuf::from("/h/.local/share/cleanping")
        );
    }

    #[test]
    fn a_relative_home_is_not_trusted() {
        let err = config_dir_from(env(&[("HOME", "rel")])).unwrap_err();
        assert!(err.to_string().contains("home directory"));
    }

    #[test]
    fn no_home_and_no_xdg_is_a_clear_error() {
        assert!(config_dir_from(env(&[]))
            .unwrap_err()
            .to_string()
            .contains("home directory"));
    }
}
