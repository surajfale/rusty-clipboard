//! clipd - background clipboard capture daemon.

mod clipboard;
mod config;
mod db;
mod ipc;
mod model;
mod service;

use anyhow::Result;
use tokio::signal;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .init();

    tracing::info!("clipd starting up");

    let config = config::Config::load()?;
    let service = service::ClipdService::bootstrap(config).await?;

    let service_task = service.run();

    tokio::select! {
        res = service_task => {
            if let Err(err) = res {
                tracing::error!(%err, "clipd service exited with error");
            }
        }
        _ = signal::ctrl_c() => {
            tracing::info!("shutdown signal received");
        }
    }

    tracing::info!("clipd shutting down");
    Ok(())
}

