use crate::domain::RecordType;
use crate::reference::ResolvedReference;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Parameters for filtering full-text search results.
#[derive(Debug, Clone, Default)]
pub struct SearchFilter {
    pub record_type: Option<RecordType>,
    pub limit: Option<usize>,
}

/// A single matched record returned from full-text search.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub record_id: Uuid,
    pub record_type: RecordType,
    pub title: String,
    pub snippet: String,
    pub score: f64,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<String>,
    pub references: Vec<ResolvedReference>,
}
