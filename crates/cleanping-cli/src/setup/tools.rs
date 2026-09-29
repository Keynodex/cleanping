//! The wizard's contact with the outside world, behind one interface so tests can fake it:
//! asking a local model server, downloading a model with Ollama, and testing a provider.

use std::process::Command;

use cleanping_core::application::connection_check::CheckOutcome;
use cleanping_core::domain::errors::Result;
use cleanping_core::domain::local_server::LocalServer;
use cleanping_core::domain::models::Credential;
use cleanping_core::infrastructure::ollama;

use crate::connection;
use crate::services::Services;

pub trait Tools {
    /// What answers at a local provider's address; `None` when the address is not this machine.
    fn probe(&self, api_url: &str) -> Option<LocalServer>;
    /// Download a model with Ollama's own `ollama pull`, showing its progress. True on success.
    fn pull(&self, model: &str) -> bool;
    /// One tiny request to check the provider.
    fn test(&self, services: &Services, credential: &Credential) -> Result<CheckOutcome>;
}

pub struct RealTools;

impl Tools for RealTools {
    fn probe(&self, api_url: &str) -> Option<LocalServer> {
        ollama::probe(api_url)
    }

    fn pull(&self, model: &str) -> bool {
        // The model is one argument, never part of a shell command line.
        Command::new("ollama")
            .arg("pull")
            .arg(model)
            .status()
            .is_ok_and(|status| status.success())
    }

    fn test(&self, services: &Services, credential: &Credential) -> Result<CheckOutcome> {
        connection::test(services, credential)
    }
}
