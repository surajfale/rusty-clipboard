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
                // Set the clipboard so the text is available for pasting
                set_clipboard(contents)?;
                tracing::info!("Set clipboard with {} chars", contents.len());
                Ok(())
            }
            PasteMethod::Stdout => {
                print!("{contents}");
                Ok(())
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn set_clipboard(text: &str) -> Result<()> {
    use clipboard_win::{formats, set_clipboard as set_clip};
    set_clip(formats::Unicode, text)
        .map_err(|e| anyhow::anyhow!("failed to set clipboard: {:?}", e))
}

#[cfg(not(target_os = "windows"))]
fn set_clipboard(_text: &str) -> Result<()> {
    anyhow::bail!("Clipboard setting is only supported on Windows")
}

