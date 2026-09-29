//! SQLite adapters for the repository ports. No business rules here.
//!
//! - `credentials`: saved provider endpoints
//! - `runs`: the rewrite history
//! - `settings`: the saved system prompt and small key/value state

/// A repository is a handle on the database; each opens a short-lived connection per call.
macro_rules! repository {
    ($($name:ident),*) => {$(
        /// SQLite adapter for one repository port. It holds only the database handle; every call
        /// opens its own short-lived connection.
        pub struct $name {
            db: crate::infrastructure::sqlite_db::Database,
        }

        impl $name {
            /// Build the repository on `db`, which must already be migrated
            /// ([`Database::migrate`](crate::infrastructure::sqlite_db::Database::migrate)).
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
