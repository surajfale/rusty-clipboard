//! Orchestrates clipboard capture, persistence, and IPC server.

use anyhow::{Error, Result};
use tokio::sync::mpsc;

use crate::clipboard::ClipboardWatcher;
use crate::config::Config;
use crate::db::Database;
use crate::ipc::Server;
use crate::model::Entry;

pub struct ClipdService {
    clipboard: ClipboardWatcher,
    db: Database,
    server: Server,
}

impl ClipdService {
    pub async fn bootstrap(config: Config) -> Result<Self> {
        let db = Database::open(config.db_path.clone(), config.max_entries)?;
        let server = Server::new(config.pipe_name.clone(), db.clone());

        Ok(Self {
            clipboard: ClipboardWatcher::new(),
            db,
            server,
        })
    }

    pub async fn run(self) -> Result<()> {
        let (entry_tx, entry_rx) = mpsc::channel::<Entry>(256);
        let Self {
            clipboard,
            db,
            server,
        } = self;

        tokio::try_join!(
            clipboard.run(entry_tx.clone()),
            async move {
                let mut entry_rx = entry_rx;
                drop(entry_tx);
                while let Some(entry) = entry_rx.recv().await {
                    db.insert_entry(&entry)?;
                }
                Ok::<(), Error>(())
            },
            async move { server.run().await },
        )?;

        Ok(())
    }
}

