//! One edit request that can run in the background while the screen waits for it.
//! Never saved to the history: the text you edit is yours alone.

use std::sync::mpsc::{self, Receiver};

use cleanping_core::application::polisher::PolishText;
use cleanping_core::domain::errors::{CleanpingError, Result};
use cleanping_core::domain::models::Credential;
use cleanping_core::infrastructure::http_rewriter::OpenAiRewriter;
use cleanping_core::infrastructure::secrets_file::JsonSecretStore;

use crate::hints;
use crate::no_history::NoHistory;
use crate::services::Services;

pub struct Job {
    text: String,
    instructions: String,
    credential: Credential,
    secrets: JsonSecretStore,
}

impl Job {
    pub fn prepare(services: &Services, text: &str, credential: Credential) -> Result<Self> {
        Ok(Self {
            text: text.to_string(),
            instructions: services.prompts.current()?,
            credential,
            secrets: services.secrets(),
        })
    }

    /// The edited text, or why there is none.
    pub fn run(self) -> Result<String> {
        let polisher = PolishText::new(OpenAiRewriter::default(), NoHistory, self.secrets);
        let result = polisher
            .run(&self.text, &self.instructions, &self.credential)
            .map_err(|error| hints::with_add_hint(error, &self.credential))?;
        match (result.output_text, result.error_message) {
            (Some(output), _) => Ok(output),
            (None, message) => Err(CleanpingError::Rewrite(
                message.unwrap_or_else(|| "The API returned nothing.".into()),
            )),
        }
    }

    pub fn spawn(self) -> Receiver<Result<String>> {
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = sender.send(self.run());
        });
        receiver
    }
}
