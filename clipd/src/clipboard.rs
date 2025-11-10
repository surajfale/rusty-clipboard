//! Clipboard listener and normalization.

use anyhow::Result;
use chrono::Utc;
use sha2::{Digest, Sha256};
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration};
use windows::Win32::Foundation::{HWND, HGLOBAL, CloseHandle};
use windows::Win32::System::DataExchange::{
    CloseClipboard, GetClipboardData, GetClipboardSequenceNumber, IsClipboardFormatAvailable, OpenClipboard,
};
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
use windows::Win32::System::Ole::{CF_UNICODETEXT, CF_DIB};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

use crate::model::{Entry, EntryKind};

/// Watches the Windows clipboard for changes and forwards normalized entries.
#[derive(Debug, Clone)]
pub struct ClipboardWatcher;

impl ClipboardWatcher {
    pub fn new() -> Self {
        Self
    }

    /// Start listening to clipboard changes using polling.
    /// This uses GetClipboardSequenceNumber to detect changes efficiently.
    pub async fn run(self, tx: Sender<Entry>) -> Result<()> {
        tracing::info!("starting clipboard watcher with polling strategy");
        
        let mut last_sequence: u32 = 0;
        let mut last_hash: Option<String> = None;
        
        loop {
            // Check if clipboard has changed
            let current_sequence = unsafe { GetClipboardSequenceNumber() };
            
            if current_sequence != last_sequence && current_sequence != 0 {
                last_sequence = current_sequence;
                tracing::debug!("clipboard sequence changed to {}", current_sequence);
                
                // Try to read in priority order: image, RTF, then text
                let entry_opt = read_clipboard_image()
                    .ok()
                    .flatten()
                    .or_else(|| read_clipboard_rtf().ok().flatten())
                    .or_else(|| {
                        read_clipboard_text().ok().flatten().map(|(text, _)| Entry {
                            id: None,
                            created_at: Utc::now(),
                            kind: EntryKind::Text,
                            text: Some(text.clone()),
                            data: None,
                            bytes_len: text.len(),
                            hash: hash_data(text.as_bytes()),
                            source_process: None,
                            tags: Vec::new(),
                        })
                    });
                
                if let Some(mut entry) = entry_opt {
                    // Skip if content hash is the same
                    if Some(&entry.hash) != last_hash.as_ref() {
                        last_hash = Some(entry.hash.clone());
                        
                        // Try to get the source process
                        entry.source_process = get_foreground_process_name();
                        
                        let bytes = entry.bytes_len;
                        let kind = entry.kind.clone();
                        let process = entry.source_process.clone();
                        if let Err(e) = tx.send(entry).await {
                            tracing::error!("failed to send clipboard entry: {}", e);
                        } else {
                            tracing::info!(
                                "captured clipboard {:?} ({} bytes) from {:?}", 
                                kind, bytes, process
                            );
                        }
                    }
                } else {
                    tracing::debug!("clipboard contains no supported content");
                }
            }
            
            // Poll every 250ms - this is efficient and responsive
            sleep(Duration::from_millis(250)).await;
        }
    }
}

/// Read text from the Windows clipboard
fn read_clipboard_text() -> Result<Option<(String, Vec<u8>)>> {
    unsafe {
        if let Err(_) = OpenClipboard(HWND::default()) {
            // Clipboard might be locked by another process, this is normal
            return Ok(None);
        }

        let result = (|| -> Result<Option<(String, Vec<u8>)>> {
            let handle = match GetClipboardData(CF_UNICODETEXT.0 as u32) {
                Ok(h) => h,
                Err(_) => return Ok(None),
            };
            
            if handle.is_invalid() {
                return Ok(None);
            }

            // Cast HANDLE to HGLOBAL for GlobalLock
            let hglobal = HGLOBAL(handle.0);
            let ptr = GlobalLock(hglobal) as *const u16;
            if ptr.is_null() {
                return Ok(None);
            }

            // Find the null terminator
            let mut len = 0;
            while *ptr.offset(len) != 0 {
                len += 1;
            }

            let slice = std::slice::from_raw_parts(ptr, len as usize);
            let text = String::from_utf16_lossy(slice);
            let bytes = text.as_bytes().to_vec();

            let _ = GlobalUnlock(hglobal);

            Ok(Some((text, bytes)))
        })();

        let _ = CloseClipboard();

        result
    }
}

/// Read image from the Windows clipboard (CF_DIB format)
fn read_clipboard_image() -> Result<Option<Entry>> {
    unsafe {
        // Check if image format is available
        if IsClipboardFormatAvailable(CF_DIB.0 as u32).is_err() {
            return Ok(None);
        }
        
        if let Err(_) = OpenClipboard(HWND::default()) {
            return Ok(None);
        }

        let result = (|| -> Result<Option<Entry>> {
            let handle = match GetClipboardData(CF_DIB.0 as u32) {
                Ok(h) => h,
                Err(_) => return Ok(None),
            };
            
            if handle.is_invalid() {
                return Ok(None);
            }

            let hglobal = HGLOBAL(handle.0);
            let ptr = GlobalLock(hglobal) as *const u8;
            if ptr.is_null() {
                return Ok(None);
            }

            let size = GlobalSize(hglobal);
            let data = std::slice::from_raw_parts(ptr, size).to_vec();

            let _ = GlobalUnlock(hglobal);

            let hash = hash_data(&data);
            let bytes_len = data.len();

            Ok(Some(Entry {
                id: None,
                created_at: Utc::now(),
                kind: EntryKind::Image,
                text: Some(format!("<image {} bytes>", bytes_len)),
                data: Some(data),
                bytes_len,
                hash,
                source_process: None,
                tags: Vec::new(),
            }))
        })();

        let _ = CloseClipboard();

        result
    }
}

/// Read RTF from the Windows clipboard
fn read_clipboard_rtf() -> Result<Option<Entry>> {
    unsafe {
        use windows::Win32::System::DataExchange::RegisterClipboardFormatW;
        use windows::core::PCWSTR;
        
        // Register RTF format
        let format_name: Vec<u16> = "Rich Text Format\0".encode_utf16().collect();
        let rtf_format = RegisterClipboardFormatW(PCWSTR(format_name.as_ptr()));
        
        if rtf_format == 0 {
            return Ok(None);
        }
        
        // Check if RTF format is available
        if IsClipboardFormatAvailable(rtf_format).is_err() {
            return Ok(None);
        }
        
        if let Err(_) = OpenClipboard(HWND::default()) {
            return Ok(None);
        }

        let result = (|| -> Result<Option<Entry>> {
            let handle = match GetClipboardData(rtf_format) {
                Ok(h) => h,
                Err(_) => return Ok(None),
            };
            
            if handle.is_invalid() {
                return Ok(None);
            }

            let hglobal = HGLOBAL(handle.0);
            let ptr = GlobalLock(hglobal) as *const u8;
            if ptr.is_null() {
                return Ok(None);
            }

            let size = GlobalSize(hglobal);
            let data = std::slice::from_raw_parts(ptr, size).to_vec();

            let _ = GlobalUnlock(hglobal);

            let hash = hash_data(&data);
            let bytes_len = data.len();
            
            // Try to convert to text for preview
            let preview_text = String::from_utf8_lossy(&data).to_string();
            let preview = if preview_text.len() > 100 {
                format!("{} ...", &preview_text[..100])
            } else {
                preview_text
            };

            Ok(Some(Entry {
                id: None,
                created_at: Utc::now(),
                kind: EntryKind::Rtf,
                text: Some(format!("<rtf {} bytes> {}", bytes_len, preview)),
                data: Some(data),
                bytes_len,
                hash,
                source_process: None,
                tags: Vec::new(),
            }))
        })();

        let _ = CloseClipboard();

        result
    }
}

/// Get the name of the foreground process
fn get_foreground_process_name() -> Option<String> {
    unsafe {
        // Get the foreground window
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }

        // Get the process ID
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
        if process_id == 0 {
            return None;
        }

        // Open the process
        let process_handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) {
            Ok(handle) => handle,
            Err(_) => return None,
        };

        // Get the process name
        use windows::core::PWSTR;
        let mut buffer = vec![0u16; 260];
        let mut size = buffer.len() as u32;
        
        let result = QueryFullProcessImageNameW(
            process_handle, 
            PROCESS_NAME_FORMAT(0), 
            PWSTR(buffer.as_mut_ptr()), 
            &mut size
        );
        let _: () = CloseHandle(process_handle).map(|_| ()).unwrap_or(());

        if result.is_err() {
            return None;
        }

        // Convert the path to a string and extract just the filename
        let path = String::from_utf16_lossy(&buffer[..size as usize]);
        let filename = path.rsplit('\\').next().unwrap_or(&path);
        Some(filename.to_string())
    }
}

/// Hash data using SHA256 for deduplication
fn hash_data(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:x}", result)
}

