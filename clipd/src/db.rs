//! SQLite persistence layer.

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use parking_lot::Mutex;
use rusqlite::{params, Connection};

use crate::model::{Entry, EntryKind};

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    max_entries: usize,
}

impl Database {
    pub fn open(path: PathBuf, max_entries: usize) -> Result<Self> {
        tracing::info!("opening sqlite database at {} (max_entries: {})", path.display(), max_entries);
        
        let conn = Connection::open(&path)
            .with_context(|| format!("failed to open database at {}", path.display()))?;
        
        // Enable WAL mode for better concurrency
        conn.pragma_update(None, "journal_mode", "WAL")?;
        
        // Create schema
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at TEXT NOT NULL,
                kind TEXT NOT NULL,
                text TEXT,
                data BLOB,
                bytes_len INTEGER NOT NULL,
                hash TEXT NOT NULL UNIQUE,
                source_process TEXT,
                tags TEXT
            );
            
            CREATE INDEX IF NOT EXISTS idx_created_at ON entries(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_hash ON entries(hash);
            "#,
        )?;
        
        tracing::info!("database schema initialized");
        
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            max_entries,
        })
    }

    pub fn insert_entry(&self, entry: &Entry) -> Result<()> {
        let conn = self.conn.lock();
        
        // Check if entry with this hash already exists
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM entries WHERE hash = ?1",
                params![&entry.hash],
                |_| Ok(true),
            )
            .unwrap_or(false);
        
        if exists {
            tracing::debug!(hash = %entry.hash, "skipping duplicate entry");
            return Ok(());
        }
        
        let kind_str = match entry.kind {
            EntryKind::Text => "text",
            EntryKind::Url => "url",
            EntryKind::Image => "image",
            EntryKind::Rtf => "rtf",
        };
        
        let tags_json = serde_json::to_string(&entry.tags)?;
        
        conn.execute(
            r#"
            INSERT INTO entries (created_at, kind, text, data, bytes_len, hash, source_process, tags)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                entry.created_at.to_rfc3339(),
                kind_str,
                &entry.text,
                &entry.data,
                entry.bytes_len as i64,
                &entry.hash,
                &entry.source_process,
                tags_json,
            ],
        )?;
        
        tracing::info!(hash = %entry.hash, "inserted new entry");
        
        // Release the lock before calling cleanup
        drop(conn);
        
        // Clean up old entries if we've exceeded the limit
        self.cleanup_old_entries()?;
        
        Ok(())
    }

    pub fn list_recent(&self, limit: usize) -> Result<Vec<Entry>> {
        let conn = self.conn.lock();
        
        let mut stmt = conn.prepare(
            r#"
            SELECT id, created_at, kind, text, data, bytes_len, hash, source_process, tags
            FROM entries
            ORDER BY created_at DESC
            LIMIT ?1
            "#,
        )?;
        
        let entries = stmt
            .query_map(params![limit as i64], |row| {
                self.entry_from_row(row)
            })?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(entries)
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<Entry>> {
        let conn = self.conn.lock();
        
        let search_pattern = format!("%{}%", query);
        
        let mut stmt = conn.prepare(
            r#"
            SELECT id, created_at, kind, text, data, bytes_len, hash, source_process, tags
            FROM entries
            WHERE text LIKE ?1 OR tags LIKE ?1
            ORDER BY created_at DESC
            LIMIT ?2
            "#,
        )?;
        
        let entries = stmt
            .query_map(params![search_pattern, limit as i64], |row| {
                self.entry_from_row(row)
            })?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(entries)
    }

    pub fn add_tag(&self, id: u64, tag: &str) -> Result<()> {
        let conn = self.conn.lock();
        
        // Get current tags
        let current_tags: String = conn.query_row(
            "SELECT tags FROM entries WHERE id = ?1",
            params![id as i64],
            |row| row.get(0),
        )?;
        
        let mut tags: Vec<String> = serde_json::from_str(&current_tags).unwrap_or_default();
        
        // Add tag if not already present
        if !tags.contains(&tag.to_string()) {
            tags.push(tag.to_string());
            let tags_json = serde_json::to_string(&tags)?;
            
            conn.execute(
                "UPDATE entries SET tags = ?1 WHERE id = ?2",
                params![tags_json, id as i64],
            )?;
            
            tracing::info!(id, tag, "tag added to entry");
        }
        
        Ok(())
    }

    pub fn remove_tag(&self, id: u64, tag: &str) -> Result<()> {
        let conn = self.conn.lock();
        
        // Get current tags
        let current_tags: String = conn.query_row(
            "SELECT tags FROM entries WHERE id = ?1",
            params![id as i64],
            |row| row.get(0),
        )?;
        
        let mut tags: Vec<String> = serde_json::from_str(&current_tags).unwrap_or_default();
        
        // Remove tag if present
        tags.retain(|t| t != tag);
        let tags_json = serde_json::to_string(&tags)?;
        
        conn.execute(
            "UPDATE entries SET tags = ?1 WHERE id = ?2",
            params![tags_json, id as i64],
        )?;
        
        tracing::info!(id, tag, "tag removed from entry");
        Ok(())
    }

    /// Remove old entries if the database exceeds max_entries
    fn cleanup_old_entries(&self) -> Result<()> {
        let conn = self.conn.lock();
        
        // Count total entries
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM entries",
            [],
            |row| row.get(0),
        )?;
        
        if count as usize > self.max_entries {
            let to_delete = count as usize - self.max_entries;
            
            conn.execute(
                r#"
                DELETE FROM entries WHERE id IN (
                    SELECT id FROM entries 
                    ORDER BY created_at ASC 
                    LIMIT ?1
                )
                "#,
                params![to_delete as i64],
            )?;
            
            tracing::info!(
                deleted = to_delete, 
                remaining = self.max_entries,
                "cleaned up old entries"
            );
        }
        
        Ok(())
    }

    /// Export all entries to a JSON file
    pub fn export_to_json(&self, path: &str) -> Result<()> {
        let conn = self.conn.lock();
        
        let mut stmt = conn.prepare(
            r#"
            SELECT id, created_at, kind, text, data, bytes_len, hash, source_process, tags
            FROM entries
            ORDER BY created_at ASC
            "#,
        )?;
        
        let entries = stmt
            .query_map([], |row| {
                self.entry_from_row(row)
            })?
            .collect::<Result<Vec<_>, _>>()?;
        
        drop(stmt);
        drop(conn);
        
        let file = File::create(path)
            .with_context(|| format!("failed to create export file: {}", path))?;
        let writer = BufWriter::new(file);
        
        serde_json::to_writer_pretty(writer, &entries)
            .with_context(|| "failed to write JSON")?;
        
        tracing::info!(count = entries.len(), "exported entries to {}", path);
        Ok(())
    }

    /// Import entries from a JSON file
    pub fn import_from_json(&self, path: &str) -> Result<()> {
        let file = File::open(path)
            .with_context(|| format!("failed to open import file: {}", path))?;
        let reader = BufReader::new(file);
        
        let entries: Vec<Entry> = serde_json::from_reader(reader)
            .with_context(|| "failed to parse JSON")?;
        
        let mut imported = 0;
        let mut skipped = 0;
        
        for entry in entries {
            let conn = self.conn.lock();
            
            // Check if entry with this hash already exists
            let exists: bool = conn
                .query_row(
                    "SELECT 1 FROM entries WHERE hash = ?1",
                    params![&entry.hash],
                    |_| Ok(true),
                )
                .unwrap_or(false);
            
            if exists {
                skipped += 1;
                drop(conn);
                continue;
            }
            
            let kind_str = match entry.kind {
                EntryKind::Text => "text",
                EntryKind::Url => "url",
                EntryKind::Image => "image",
                EntryKind::Rtf => "rtf",
            };
            
            let tags_json = serde_json::to_string(&entry.tags)?;
            
            conn.execute(
                r#"
                INSERT INTO entries (created_at, kind, text, data, bytes_len, hash, source_process, tags)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#,
                params![
                    entry.created_at.to_rfc3339(),
                    kind_str,
                    &entry.text,
                    &entry.data,
                    entry.bytes_len as i64,
                    &entry.hash,
                    &entry.source_process,
                    tags_json,
                ],
            )?;
            
            imported += 1;
            drop(conn);
        }
        
        tracing::info!(
            imported, 
            skipped, 
            "imported entries from {}", 
            path
        );
        
        Ok(())
    }

    fn entry_from_row(&self, row: &rusqlite::Row) -> rusqlite::Result<Entry> {
        let kind_str: String = row.get(2)?;
        let kind = match kind_str.as_str() {
            "text" => EntryKind::Text,
            "url" => EntryKind::Url,
            "image" => EntryKind::Image,
            "rtf" => EntryKind::Rtf,
            _ => EntryKind::Text,
        };
        
        let created_at_str: String = row.get(1)?;
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());
        
        let tags_json: String = row.get(8)?;
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
        
        Ok(Entry {
            id: Some(row.get(0)?),
            created_at,
            kind,
            text: row.get(3)?,
            data: row.get(4)?,
            bytes_len: row.get::<_, i64>(5)? as usize,
            hash: row.get(6)?,
            source_process: row.get(7)?,
            tags,
        })
    }
}

