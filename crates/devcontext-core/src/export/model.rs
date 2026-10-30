//! Data models and interchange schemas for DevContext export, import, and data safety.

use crate::error::{CoreError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

pub const SCHEMA_VERSION_1: &str = "1.0.0";

/// Supported export format targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExportFormat {
    Json,
    Markdown,
    Zip,
}

impl fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Markdown => write!(f, "markdown"),
            Self::Zip => write!(f, "zip"),
        }
    }
}

impl FromStr for ExportFormat {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_lowercase().as_str() {
            "json" | "export-v1.json" => Ok(Self::Json),
            "markdown" | "md" => Ok(Self::Markdown),
            "zip" | "archive" | "bundle" => Ok(Self::Zip),
            _ => Err(CoreError::Validation(format!(
                "Invalid export format '{}'. Expected 'json', 'markdown', or 'zip'.",
                s
            ))),
        }
    }
}

/// Strategy for resolving incoming records with colliding UUIDs during import.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConflictStrategy {
    Skip,
    Overwrite,
    GenerateNewId,
}

impl fmt::Display for ConflictStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Skip => write!(f, "skip"),
            Self::Overwrite => write!(f, "overwrite"),
            Self::GenerateNewId => write!(f, "generate-new-id"),
        }
    }
}

impl FromStr for ConflictStrategy {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_lowercase().as_str() {
            "skip" => Ok(Self::Skip),
            "overwrite" => Ok(Self::Overwrite),
            "generate-new-id" | "new-id" | "rename" => Ok(Self::GenerateNewId),
            _ => Err(CoreError::Validation(format!(
                "Invalid conflict strategy '{}'. Expected 'skip', 'overwrite', or 'generate-new-id'.",
                s
            ))),
        }
    }
}

/// Top-level interchange document conforming to `schemas/export-v1.schema.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportBundle {
    pub schema_version: String,
    pub exported_at: String,
    pub generator: GeneratorInfo,
    pub project: ExportProject,
    pub records: Vec<ExportRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneratorInfo {
    pub name: String,
    pub version: String,
}

impl Default for GeneratorInfo {
    fn default() -> Self {
        Self {
            name: "devcontext-core".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportProject {
    pub id: Uuid,
    pub name: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcs: Option<ExportVcs>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportVcs {
    #[serde(rename = "type")]
    pub vcs_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_identifier: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportRecord {
    pub id: Uuid,
    pub record_type: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<ExportReference>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportReference {
    pub id: Uuid,
    pub ref_type: String,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

// Typed payload structures for serialization/deserialization validation

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotePayload {
    pub content_markdown: String,
    pub category: Option<String>,
    #[serde(default)]
    pub is_pinned: bool,
    #[serde(default)]
    pub is_archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecisionPayload {
    pub status: String,
    pub context: String,
    pub decision: String,
    pub consequences: String,
    pub alternatives_considered: Option<String>,
    pub superseded_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MilestonePayload {
    pub status: String,
    pub target_date: Option<String>,
    pub completed_date: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntryPayload {
    pub event_date: String,
    pub summary: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SavedCommandPayload {
    pub command_text: String,
    pub explanation: Option<String>,
    pub working_directory: Option<String>,
}

/// Options for export operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub output_path: Option<std::path::PathBuf>,
    pub redact_paths: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Json,
            output_path: None,
            redact_paths: false,
        }
    }
}

/// Summary preview before creating an export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPreview {
    pub project_name: String,
    pub notes_count: usize,
    pub decisions_count: usize,
    pub milestones_count: usize,
    pub logs_count: usize,
    pub commands_count: usize,
    pub references_count: usize,
    pub total_records: usize,
    pub redact_paths: bool,
}

/// Options for import operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    pub on_conflict: ConflictStrategy,
    pub create_backup: bool,
    pub dry_run: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            on_conflict: ConflictStrategy::Skip,
            create_backup: true,
            dry_run: false,
        }
    }
}

/// Outcome of an import operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub dry_run: bool,
    pub total_incoming: usize,
    pub imported_count: usize,
    pub skipped_count: usize,
    pub overwritten_count: usize,
    pub conflicts_count: usize,
    pub backup_path: Option<String>,
}

/// Detailed inspection report for an archive, JSON bundle, or existing database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionReport {
    pub source_type: String,
    pub schema_version: Option<String>,
    pub project_name: String,
    pub project_id: Uuid,
    pub notes_count: usize,
    pub decisions_count: usize,
    pub milestones_count: usize,
    pub logs_count: usize,
    pub commands_count: usize,
    pub references_count: usize,
    pub total_records: usize,
    pub conflicts_with_existing: usize,
}

/// Outcome of complete project data deletion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteResult {
    pub project_name: String,
    pub deleted_records_count: usize,
    pub backup_path: Option<String>,
}
