//! Abstractions for sending paste actions to the active terminal.

use anyhow::Result;

#[allow(dead_code)]
pub enum PasteMethod {
    SendInput,
    Stdout,
}

pub struct PasteEngine {
    method: PasteMethod,
}

impl PasteEngine {
    pub fn new(method: PasteMethod) -> Self {
        Self { method }
    }

    pub fn paste(&self, contents: &str) -> Result<()> {
        match self.method {
            PasteMethod::SendInput => {
                tracing::info!("SendInput paste stub ({} chars)", contents.len());
                Ok(())
            }
            PasteMethod::Stdout => {
                print!("{contents}");
                Ok(())
            }
        }
    }
}

