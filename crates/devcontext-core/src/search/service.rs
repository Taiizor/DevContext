use super::model::{SearchFilter, SearchResult};
use crate::error::{CoreError, Result};
use crate::reference::ReferenceResolver;
use crate::storage::SqliteStorage;
use std::path::Path;
use uuid::Uuid;

/// Default limit for search results if unspecified.
pub const DEFAULT_SEARCH_LIMIT: usize = 50;

/// Orchestrates full-text search across all authored DevContext records.
pub struct SearchService;

impl SearchService {
    /// Executes full-text search against the SQLite FTS5 index.
    pub fn search(
        storage: &SqliteStorage,
        project_id: Uuid,
        project_root: &Path,
        query: &str,
        filter: &SearchFilter,
    ) -> Result<Vec<SearchResult>> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }

        let limit = filter.limit.unwrap_or(DEFAULT_SEARCH_LIMIT);

        // Normalize Turkish dotless-i ('ı') to 'i' so search behaves symmetrically
        // across Turkish case folding and diacritics in SQLite unicode61.
        let normalized = trimmed.replace('ı', "i");

        // Attempt user's query; if SQLite returns an FTS5 syntax error (e.g. unclosed quote or operator),
        // fallback to sanitized token query.
        let raw_matches =
            match storage.search_raw(project_id, &normalized, filter.record_type, limit) {
                Ok(matches) => matches,
                Err(CoreError::Sqlite(_)) => {
                    let sanitized = Self::sanitize_fts5_query(&normalized);
                    if sanitized.is_empty() {
                        return Ok(Vec::new());
                    }
                    storage
                        .search_raw(project_id, &sanitized, filter.record_type, limit)
                        .unwrap_or_default()
                }
                Err(e) => return Err(e),
            };

        let mut results = Vec::with_capacity(raw_matches.len());

        for (record_id, record_type, title, snippet, score, created_at, updated_at, tags) in
            raw_matches
        {
            // Load and resolve any attached references for navigation
            let refs = storage
                .list_references_for_record(record_id)
                .unwrap_or_default();
            let resolved_refs = ReferenceResolver::resolve_all(&refs, project_root, Some(storage));

            results.push(SearchResult {
                record_id,
                record_type,
                title,
                snippet,
                score,
                created_at,
                updated_at,
                tags,
                references: resolved_refs,
            });
        }

        Ok(results)
    }

    /// Rebuilds the search index from all active records in the database.
    pub fn rebuild_index(storage: &SqliteStorage) -> Result<usize> {
        storage.rebuild_search_index()
    }

    /// Sanitizes an input query into quoted tokens to protect against FTS5 syntax errors.
    fn sanitize_fts5_query(input: &str) -> String {
        let mut tokens = Vec::new();
        for word in input.split_whitespace() {
            // Keep alphanumeric, dash, and underscore characters
            let clean: String = word
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect();
            if !clean.is_empty() {
                tokens.push(format!("\"{}\"", clean));
            }
        }
        tokens.join(" ")
    }
}
