//! The edit screen's loop: show the view, read keys, and pick up the reply when it arrives.

use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

use cleanping_core::domain::command_change::{command_changes, summary};
use cleanping_core::domain::diff::highlight_changes;
use cleanping_core::domain::errors::Result;

use super::job::Job;
use super::terminal::{Input, Screen};
use super::view::{Action, Phase, Review, View};

const TICK: Duration = Duration::from_millis(100);

pub enum Outcome {
    Accept(String),
    Keep,
}

/// The reply, or the reason there is none, as a screen state.
fn phase_for(reply: Result<String>, original: &str) -> Phase {
    match reply {
        Ok(edited) => Phase::Review(Review {
            diff: highlight_changes(original, &edited),
            command_warning: summary(&command_changes(original, &edited)),
            edited,
            showing_original: false,
        }),
        Err(error) => Phase::Failed(error.to_string()),
    }
}

/// Start the request; if it cannot even start, say why.
fn start(prepared: Result<Job>, pending: &mut Option<Receiver<Result<String>>>) -> Phase {
    match prepared {
        Ok(job) => {
            *pending = Some(job.spawn());
            Phase::Waiting
        }
        Err(error) => Phase::Failed(error.to_string()),
    }
}

/// `refused` names the kind of secret the text seems to hold; nothing is sent until the
/// user chooses to send it anyway.
pub fn run(
    screen: &mut Screen,
    text: &str,
    provider: String,
    refused: Option<&'static str>,
    prepared: Result<Job>,
) -> Result<Outcome> {
    let mut prepared = Some(prepared);
    let mut pending = None;
    let phase = match refused {
        Some(kind) => Phase::Refused(kind),
        None => start(prepared.take().expect("set above"), &mut pending),
    };
    let mut view = View::new(text.to_string(), provider, phase);
    let mut dirty = true;
    loop {
        if dirty {
            let (width, height) = screen.size();
            screen.paint(&view.render(width, height))?;
            dirty = false;
        }
        match screen.next_input(TICK)? {
            Input::Key(key) => {
                let page = screen.size().1.saturating_sub(3).max(1);
                match view.on_key(key, page) {
                    Action::Accept(edited) => return Ok(Outcome::Accept(edited)),
                    Action::KeepOriginal => return Ok(Outcome::Keep),
                    Action::SendAnyway => {
                        if let Some(prepared) = prepared.take() {
                            view.phase = start(prepared, &mut pending);
                        }
                    }
                    Action::Continue => {}
                }
                dirty = true;
            }
            Input::Resized => dirty = true,
            Input::Nothing => {}
        }
        if let Some(receiver) = &pending {
            match receiver.try_recv() {
                Ok(reply) => view.phase = phase_for(reply, text),
                Err(TryRecvError::Empty) => continue,
                Err(TryRecvError::Disconnected) => {
                    view.phase = Phase::Failed("The edit stopped before it finished.".into());
                }
            }
            pending = None;
            dirty = true;
        }
    }
}
