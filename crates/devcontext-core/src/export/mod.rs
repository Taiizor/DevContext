//! Export, import, inspection, and data safety module for DevContext.
//!
//! Provides versioned JSON serialization (conforming to `schemas/export-v1.schema.json`),
//! standalone Markdown generation with YAML frontmatter, portable ZIP bundle packaging and extraction,
//! path sanitization/redaction, and safe transactional import/recovery.

pub mod archive;
pub mod markdown;
pub mod model;
pub mod sanitization;
pub mod service;

pub use archive::{create_zip_archive, extract_zip_safely, read_bundle_from_zip};
pub use markdown::generate_markdown_export;
pub use model::{
    ConflictStrategy, DecisionPayload, DeleteResult, ExportBundle, ExportFormat, ExportOptions,
    ExportPreview, ExportProject, ExportRecord, ExportReference, ExportVcs, GeneratorInfo,
    ImportOptions, ImportResult, InspectionReport, LogEntryPayload, MilestonePayload, NotePayload,
    SavedCommandPayload, SCHEMA_VERSION_1,
};
pub use sanitization::{is_safe_archive_path, normalize_slashes, sanitize_target};
pub use service::ExportService;
