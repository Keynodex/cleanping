//! Core of CleanPing. Layers mirror the earlier Python version (`legacy/python`):
//! `domain` (pure rules) <- `application` (use cases + ports) <- `infrastructure` (adapters).

// A public item without a doc comment is a warning here, and CI turns warnings into errors.
#![warn(missing_docs)]

pub mod application;
pub mod domain;
pub mod infrastructure;
