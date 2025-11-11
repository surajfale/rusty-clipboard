# Quick FAQs

Short answers to the most common “how do I…?” questions for Rusty Clipboard.

## Setup & Installation

- **How do I install the toolchain?**  
  `winget install --id Rustlang.Rust.MSVC`

- **How do I clone and install the project?**  
  ```powershell
  git clone https://github.com/<you>/rusty-clipboard.git
  cd rusty-clipboard/main
  .\install.ps1
  ```

- **What does the installer do?**  
  Builds release binaries, installs to `%LOCALAPPDATA%\Programs\rusty-clipboard`, sets up PATH and environment variables, configures F12 hotkey in PowerShell, and starts the daemon.

- **Where are the installed binaries?**  
  `%LOCALAPPDATA%\Programs\rusty-clipboard\clipd.exe` and `clipctl.exe`

## Running & Stopping

- **Start the daemon (if not auto-started):** `clipd` (or `cargo run --bin clipd` for dev)
- **Run the UI client:** `clipctl` or press `F12` in PowerShell (or `cargo run --bin clipctl` for dev)
- **Stop the daemon:** `Stop-Process -Name clipd` or `Ctrl+C` if running in foreground
- **Launch UI with hotkey:** `F12` in PowerShell (configured by installer automatically)

## Configuration

- **Enable debug logs:** `$env:RUST_LOG = "clipd=debug,clipctl=debug"`
- **Change history cap:** `$env:CLIPMGR_MAX_ENTRIES = 5000`
- **Switch pipe name:** `$env:CLIPMGR_PIPE = "\\.\pipe\clipmgr-alt"` (set for both processes)

## Key Keybindings (`clipctl`)

- **Navigation:** `j`/`Down` (next), `k`/`Up` (previous), `g` (top), `G` (bottom)
- **Search:** `/` to enter search mode, type query, `Enter`/`Esc` to exit
- **Paste:** `Enter` or `l`
- **Tags:** `t` add tag, `T` remove tag
- **Import/Export:** `i` import JSON, `e` export history
- **Help:** `?` opens help screen with all keybindings (press any key to close)
- **Quit:** `q` or `Esc`

## Visual Features

- **Color Themes:** Multiple beautiful themes available:
  - **Nord** (default): Cool blues and purples with a clean, modern look
  - **Dracula**: Vibrant purples and pinks with high contrast
  - **Tokyo Night**: Deep blues with vibrant accents
  - **Gruvbox**: Warm, earthy tones with excellent readability
- **Syntax Highlighting:** Automatic code detection and highlighting for Rust, Python, JavaScript, Go, C++, Java, SQL, Bash, PHP, and more. Uses syntect library with base16-ocean.dark theme.
- **Rich Text Rendering:** Markdown-style formatting with:
  - Colored headers (`#`, `##`)
  - Styled bullet points
  - Inline code blocks with background highlighting
  - Bold text support
- **Colored Icons:** Different colors for each content type:
  - 📝 Text (cyan/blue)
  - 🔗 URLs (blue/purple)
  - 🖼️ Images (purple/pink)
  - 📄 RTF/Documents (yellow)
- **Smart Preview:** Auto-detects code snippets and applies appropriate syntax highlighting. Also detects markdown-style formatting and renders accordingly.
- **Enhanced Metadata Display:** Preview pane shows type, source process, tags, and timestamp with styled labels and values.
- **Visual Feedback:** 
  - Focused borders when in input modes
  - Styled tags with colored backgrounds
  - Mode-aware command bar with emojis (🔍 search, 🏷️ add tag, 🗑️ remove tag, 💾 export, 📥 import)
  - Help screen overlay accessible with `?`

## Common Paths & Files

- **Database:** `%LOCALAPPDATA%\clipmgr\history.db`
- **Config:** `%APPDATA%\clipmgr\config.toml` (created when custom settings are saved)
- **Exports:** location you specify (default `clipboard_export.json`)

## Troubleshooting Quick Hits

- **UI says it can’t connect:** ensure `clipd` is running and both share the same `CLIPMGR_PIPE`.
- **Pastes do nothing:** confirm the target window has focus; try running UI elevated if required.
- **History missing entries:** check logs for polling warnings; some UWP apps block clipboard listeners.
- **Import reports 0 entries:** duplicates already exist—hash-based dedupe skips them.


