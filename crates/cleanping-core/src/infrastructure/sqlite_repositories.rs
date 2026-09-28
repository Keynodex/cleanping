//! SQLite adapters for the repository ports. No business rules here.
//!
//! - `credentials`: saved provider endpoints
//! - `runs`: the rewrite history
//! - `settings`: the saved system prompt and small key/value state

/// A repository is a handle on the database; each opens a short-lived connection per call.
macro_rules! repository {
    ($($name:ident),*) => {$(
        pub struct $name {
            db: crate::infrastructure::sqlite_db::Database,
        }

        impl $name {
            pub fn new(db: crate::infrastructure::sqlite_db::Database) -> Self {
                Self { db }
            }
        }
    )*};
}

mod credentials;
mod runs;
mod settings;

pub use credentials::SqliteCredentialRepository;
pub use runs::SqliteRunRepository;
pub use settings::{SqlitePromptRepository, SqliteStateRepository};

#[cfg(test)]
#[path = "sqlite_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "sqlite_erase_tests.rs"]
mod erase_tests;
