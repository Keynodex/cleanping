//! Use cases and the ports (traits) they depend on.

pub mod app_state;
pub mod connection_check;
pub mod credentials;
pub mod history;
pub mod polisher;
pub mod ports;
pub mod prompts;
pub mod update_check;
pub mod update_text;

#[cfg(test)]
mod test_support;
