//! `cleanping setup`: a guided first run (pick the AI, save the key, pick a system prompt,
//! test the connection), and a small menu when run again.

mod ask;
mod console;
mod provider;
#[cfg(test)]
mod testing;
mod tools;
mod usage;
