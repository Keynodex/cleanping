//! `cleanping setup`: a guided first run (pick the AI, save the key, pick a system prompt,
//! test the connection), and a small menu when run again.

mod ask;
mod check;
mod console;
mod provider;
mod system_prompt;
#[cfg(test)]
mod testing;
mod tools;
mod usage;
