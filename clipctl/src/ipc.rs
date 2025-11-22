use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};

const PIPE_NAME: &str = r"\\.\pipe\clipmgr";

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

pub struct Client {
    pipe: NamedPipeClient,
}

impl Client {
    pub async fn connect() -> Result<Self> {
        let pipe = ClientOptions::new()
            .open(PIPE_NAME)
            .with_context(|| {
                format!(
                    "failed to connect to pipe {PIPE_NAME}\n\
                    This usually means:\n\
                    1. The clipd daemon is not running - start it with: cargo run --bin clipd\n\
                    2. The daemon was started with different permissions (e.g., as administrator)\n\
                    3. Check if clipd is running: Get-Process clipd"
                )
            })?;
        Ok(Self { pipe })
    }

    pub async fn send(&mut self, request: &Request) -> Result<()> {
        let payload = serde_json::to_vec(request)?;
        let len = payload.len() as u32;
        self.pipe.write_u32_le(len).await?;
        self.pipe.write_all(&payload).await?;
        self.pipe.flush().await?;
        Ok(())
    }

    pub async fn next_message(&mut self) -> Result<Response> {
        let len = self.pipe.read_u32_le().await?;
        let mut buf = vec![0u8; len as usize];
        self.pipe.read_exact(&mut buf).await?;
        Ok(serde_json::from_slice(&buf)?)
    }
}

