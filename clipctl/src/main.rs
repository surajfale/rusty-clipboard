//! clipctl - terminal UI client for clipboard manager.

mod app;
mod ipc;
mod paste;
mod syntax;
mod theme;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use tokio::runtime::Runtime;

fn main() -> Result<()> {
    let rt = Runtime::new()?;
    rt.block_on(async { run_async().await })
}

async fn run_async() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnableMouseCapture)?;

    let res = app::App::run().await;

    crossterm::execute!(stdout, DisableMouseCapture)?;
    disable_raw_mode()?;

    res
}

