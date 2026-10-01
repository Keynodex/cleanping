//! Adapters for the ports: files, SQLite, HTTP. No business rules here.

mod chat_reply;
mod chat_request;
pub mod clock;
#[cfg(test)]
mod fake_api;
pub mod github_releases;
pub mod http_rewriter;
pub mod ollama;
pub mod paths;
pub mod private_fs;
pub mod proxy_env;
pub mod secrets_file;
pub mod sqlite_db;
pub mod sqlite_repositories;
