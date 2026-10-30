use serde::{Deserialize, Serialize};

/// Strongly typed snapshot of Git commit metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitCommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: String,
    pub message: String,
}

impl GitCommitInfo {
    pub fn new(
        hash: impl Into<String>,
        author_name: impl Into<String>,
        author_email: impl Into<String>,
        timestamp: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        let full_hash = hash.into();
        let short_hash = if full_hash.len() >= 7 {
            full_hash[..7].to_string()
        } else {
            full_hash.clone()
        };

        Self {
            hash: full_hash,
            short_hash,
            author_name: author_name.into(),
            author_email: author_email.into(),
            timestamp: timestamp.into(),
            message: message.into(),
        }
    }
}
