//! Named pipe IPC server.

use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};

use crate::db::Database;
use crate::model::Entry;

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub kind: RequestKind,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestKind {
    List,
    Search { query: String },
    Paste { id: u64 },
    AddTag { id: u64, tag: String },
    RemoveTag { id: u64, tag: String },
    Export { path: String },
    Import { path: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub entries: Vec<EntrySummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntrySummary {
    pub id: u64,
    pub preview: String,
    pub created_at: String,
    pub kind: String,
    pub source_process: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Clone)]
pub struct Server {
    inner: Arc<ServerInner>,
}

struct ServerInner {
    pipe_name: String,
    db: Database,
}

impl Server {
    pub fn new(pipe_name: String, db: Database) -> Self {
        Self {
            inner: Arc::new(ServerInner { pipe_name, db }),
        }
    }

    pub async fn run(&self) -> Result<()> {
        loop {
            let pipe = self.create_pipe()?;
            if let Err(err) = pipe.connect().await {
                tracing::warn!(%err, "failed to connect named pipe client");
                continue;
            }

            let inner = self.inner.clone();
            tokio::spawn(async move {
                if let Err(err) = inner.handle_client(pipe).await {
                    tracing::warn!(%err, "client handler failed");
                }
            });
        }
    }

    fn create_pipe(&self) -> Result<NamedPipeServer> {
        ServerOptions::new()
            .create(&self.inner.pipe_name)
            .with_context(|| format!("failed to create named pipe {}", self.inner.pipe_name))
    }
}

impl ServerInner {
    async fn handle_client(&self, mut pipe: NamedPipeServer) -> Result<()> {
        tracing::info!("client connected");
        loop {
            let len = match pipe.read_u32_le().await {
                Ok(len) => len,
                Err(err) => {
                    tracing::debug!(%err, "client disconnected");
                    break;
                }
            };

            let mut buf = vec![0u8; len as usize];
            pipe.read_exact(&mut buf).await?;

            let request: Request = serde_json::from_slice(&buf)?;
            let response = self.dispatch(request).await?;

            let payload = serde_json::to_vec(&response)?;
            pipe.write_u32_le(payload.len() as u32).await?;
            pipe.write_all(&payload).await?;
            pipe.flush().await?;
        }

        Ok(())
    }

    async fn dispatch(&self, request: Request) -> Result<Response> {
        match request.kind {
            RequestKind::List => self.handle_list().await,
            RequestKind::Search { query } => self.handle_search(query).await,
            RequestKind::Paste { id } => self.handle_paste(id).await,
            RequestKind::AddTag { id, tag } => self.handle_add_tag(id, tag).await,
            RequestKind::RemoveTag { id, tag } => self.handle_remove_tag(id, tag).await,
            RequestKind::Export { path } => self.handle_export(path).await,
            RequestKind::Import { path } => self.handle_import(path).await,
        }
    }

    async fn handle_list(&self) -> Result<Response> {
        let entries = self.db.list_recent(256)?;
        Ok(Response {
            entries: entries.into_iter().map(EntrySummary::from).collect(),
        })
    }

    async fn handle_search(&self, query: String) -> Result<Response> {
        tracing::debug!(%query, "searching clipboard history");
        
        // If query is empty, return all recent entries
        let entries = if query.is_empty() {
            self.db.list_recent(256)?
        } else {
            self.db.search(&query, 256)?
        };
        
        Ok(Response {
            entries: entries.into_iter().map(EntrySummary::from).collect(),
        })
    }

    async fn handle_paste(&self, id: u64) -> Result<Response> {
        tracing::info!(id, "received paste request");
        self.handle_list().await
    }

    async fn handle_add_tag(&self, id: u64, tag: String) -> Result<Response> {
        tracing::info!(id, %tag, "adding tag to entry");
        self.db.add_tag(id, &tag)?;
        self.handle_list().await
    }

    async fn handle_remove_tag(&self, id: u64, tag: String) -> Result<Response> {
        tracing::info!(id, %tag, "removing tag from entry");
        self.db.remove_tag(id, &tag)?;
        self.handle_list().await
    }

    async fn handle_export(&self, path: String) -> Result<Response> {
        tracing::info!(%path, "exporting clipboard history");
        self.db.export_to_json(&path)?;
        self.handle_list().await
    }

    async fn handle_import(&self, path: String) -> Result<Response> {
        tracing::info!(%path, "importing clipboard history");
        self.db.import_from_json(&path)?;
        self.handle_list().await
    }
}

impl From<Entry> for EntrySummary {
    fn from(entry: Entry) -> Self {
        let kind = match entry.kind {
            crate::model::EntryKind::Text => "text",
            crate::model::EntryKind::Url => "url",
            crate::model::EntryKind::Image => "image",
            crate::model::EntryKind::Rtf => "rtf",
        };
        
        Self {
            id: entry.id.unwrap_or_default(),
            preview: entry
                .text
                .unwrap_or_else(|| "<non-text entry>".to_string()),
            created_at: entry.created_at.to_rfc3339(),
            kind: kind.to_string(),
            source_process: entry.source_process,
            tags: entry.tags,
        }
    }
}

