//! Exit codes and stderr reporting. Messages never contain secrets (the core guarantees that).

use cleanping_core::domain::errors::CleanpingError;

pub const OK: i32 = 0;

pub fn code_for(error: &CleanpingError) -> i32 {
    match error {
        CleanpingError::Validation(_) => 2,
        CleanpingError::MissingCredential(_) | CleanpingError::NotFound(_) => 3,
        CleanpingError::Rewrite(_) | CleanpingError::Storage(_) => 1,
    }
}

pub fn fail(error: &CleanpingError) -> i32 {
    crate::output::warn(&error.to_string());
    code_for(error)
}
