//! Adapters for the ports: files, SQLite, HTTP. No business rules here.

mod chat_reply;
pub mod clock;
pub mod http_rewriter;
pub mod ollama;
pub mod paths;
pub mod private_fs;
pub mod proxy_env;
pub mod secrets_file;
pub mod sqlite_db;
pub mod sqlite_repositories;
