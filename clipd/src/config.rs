//! Configuration loading for clipd.

use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;

const PIPE_NAME: &str = r"\\.\pipe\clipmgr";

#[derive(Debug, Clone)]
pub struct Config {
    pub db_path: PathBuf,
    pub pipe_name: String,
    pub max_entries: usize,
}

impl Config {
    pub fn load() -> Result<Self> {
        let dirs = ProjectDirs::from("com", "rusty-clipboard", "clipmgr")
            .context("failed to determine application directories")?;

        let mut db_path = dirs.data_local_dir().to_path_buf();
        std::fs::create_dir_all(&db_path).with_context(|| {
            format!(
                "failed to create data directory: {}",
                db_path.display()
            )
        })?;
        db_path.push("history.db");

        let pipe_name = env::var("CLIPMGR_PIPE").unwrap_or_else(|_| PIPE_NAME.to_string());
        
        let max_entries = env::var("CLIPMGR_MAX_ENTRIES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10000);

        Ok(Self { db_path, pipe_name, max_entries })
    }
}

