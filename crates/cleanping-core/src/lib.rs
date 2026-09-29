//! Core of CleanPing. Layers mirror the earlier Python version (`legacy/python`):
//! `domain` (pure rules) <- `application` (use cases + ports) <- `infrastructure` (adapters).

pub mod application;
pub mod domain;
pub mod infrastructure;
