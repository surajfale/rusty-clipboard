//! Shared data models for clipboard entries.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryKind {
    Text,
    Url,
    Image,
    Rtf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub kind: EntryKind,
    pub text: Option<String>,
    pub data: Option<Vec<u8>>,  // Binary data for images/RTF
    pub bytes_len: usize,
    pub hash: String,
    pub source_process: Option<String>,
    pub tags: Vec<String>,
}

