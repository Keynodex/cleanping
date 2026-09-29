//! Asks a local model server (Ollama) what it is and which models it has. Loopback only:
//! it never touches an address that is not this machine, never follows redirects and never
//! goes through a proxy.

use std::ffi::OsStr;
use std::path::Path;
use std::time::Duration;

use serde_json::Value;
use url::Url;

use crate::domain::local_server::LocalServer;
use crate::domain::validation::is_local_url;
use crate::infrastructure::http_rewriter::MAX_RESPONSE_BYTES;

/// A local server that does not answer this fast is treated as not running.
const TIMEOUT: Duration = Duration::from_secs(3);

/// What answers at the address of a local provider, or `None` when that address is not this
/// machine (nothing is sent in that case). `PATH` is read from the environment.
pub fn probe(api_url: &str) -> Option<LocalServer> {
    probe_with(api_url, std::env::var_os("PATH").as_deref())
}

pub fn probe_with(api_url: &str, path: Option<&OsStr>) -> Option<LocalServer> {
    if !is_local_url(api_url) {
        return None;
    }
    let tags = Url::parse(api_url.trim()).ok()?.join("/api/tags").ok()?;
    let config = ureq::Agent::config_builder()
        .max_redirects(0)
        .max_redirects_will_error(false)
        .http_status_as_error(false)
        .timeout_global(Some(TIMEOUT))
        .proxy(None) // a loopback address stays on this machine, whatever HTTP_PROXY says
        .build();
    let agent = ureq::Agent::new_with_config(config);
    let Ok(mut response) = agent.get(tags.as_str()).call() else {
        return Some(if program_on_path(path) {
            LocalServer::NotRunning
        } else {
            LocalServer::NotInstalled
        });
    };
    if !response.status().is_success() {
        return Some(LocalServer::Other);
    }
    let body = response
        .body_mut()
        .with_config()
        .limit(MAX_RESPONSE_BYTES)
        .read_to_string();
    Some(match body.ok().and_then(|text| model_names(&text)) {
        Some(models) => LocalServer::Running { models },
        None => LocalServer::Other,
    })
}

/// The model names in an Ollama `/api/tags` answer, or `None` if it is not shaped like one.
fn model_names(body: &str) -> Option<Vec<String>> {
    let value: Value = serde_json::from_str(body).ok()?;
    let models = value.get("models")?.as_array()?;
    Some(
        models
            .iter()
            .filter_map(|model| model.get("name")?.as_str().map(str::to_string))
            .collect(),
    )
}

fn program_on_path(path: Option<&OsStr>) -> bool {
    path.is_some_and(|path| std::env::split_paths(path).any(|dir| is_runnable(&dir.join("ollama"))))
}

fn is_runnable(file: &Path) -> bool {
    let Ok(meta) = file.metadata() else {
        return false;
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.is_file() && meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        meta.is_file()
    }
}

#[cfg(test)]
#[path = "ollama_tests.rs"]
mod tests;
