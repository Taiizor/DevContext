//! DevContext Core Library
//!
//! Reusable domain logic, storage primitives, Git integration, search, and export capabilities.

pub mod domain;
pub mod error;
pub mod export;
pub mod git;
pub mod search;
pub mod storage;

pub use error::{CoreError, Result};

/// Returns the semantic version of the core engine.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_version() {
        assert_eq!(version(), "0.1.0");
    }
}
