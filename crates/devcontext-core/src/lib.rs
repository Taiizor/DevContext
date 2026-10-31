//! DevContext Core Library
//!
//! Reusable domain logic, storage primitives, Git integration, search, and export capabilities.

pub mod domain;
pub mod error;
pub mod export;
pub mod git;
pub mod reference;
pub mod search;
pub mod storage;
pub mod workspace;

pub use error::{CoreError, Result};
pub use export::{
    ConflictStrategy, DeleteResult, ExportBundle, ExportFormat, ExportOptions, ExportPreview,
    ExportService, ImportOptions, ImportResult, InspectionReport,
};
pub use git::{
    GitAdapter, GitCommitInfo, TimelineEvent, TimelineEventType, TimelineFilter, TimelineService,
    TimelineView,
};
pub use reference::{ReferenceResolutionStatus, ReferenceResolver, ResolvedReference};
pub use search::{SearchFilter, SearchResult, SearchService};
pub use workspace::WorkspaceManager;

/// Returns the semantic version of the core engine.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_version() {
        assert_eq!(version(), "0.1.2");
    }
}
