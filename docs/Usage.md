# Usage

This guide covers prerequisites, setup, daily workflows, and advanced operations for Rusty Clipboard.

## Prerequisites

- Windows 11 22H2 or later
- Rust stable toolchain via `rustup` (MSVC target)
- Visual C++ Build Tools (installed automatically with the MSVC toolchain)

Verify the toolchain:

```powershell
rustup show active-toolchain
cargo --version
```

## Install & Build

### Quick Install (Recommended)

Clone the repository and run the installer:

```powershell
git clone https://github.com/<you>/rusty-clipboard.git
cd rusty-clipboard/main
.\install.ps1
```

The installer script will:
- Build release binaries (`clipd.exe` and `clipctl.exe`)
- Copy executables to `%LOCALAPPDATA%\Programs\rusty-clipboard`
- Set `RUSTY_CLIPBOARD_HOME` environment variable
- Add the installation directory to your user PATH
- Configure PowerShell profile to:
  - Auto-start `clipd` daemon on PowerShell launch
  - Bind `F12` to launch `clipctl`
- Start the daemon immediately

After installation, restart any open PowerShell sessions to pick up the profile and PATH changes.

### Manual Development Build

For development without installation:

```powershell
cargo build          # debug
cargo build --release
```

Release artifacts live under `target\release\clipd.exe` and `target\release\clipctl.exe`.

Run linting and formatting:

```powershell
cargo fmt
cargo clippy --all-targets --all-features
```

## Running the Daemon (`clipd`)

### After Installation

If you used `install.ps1`, the daemon starts automatically when you open a new PowerShell session. To check if it's running:

```powershell
Get-Process clipd
```

To manually start it (if not running):

```powershell
clipd
```

### Development Mode

Start the background service in one terminal:

```powershell
cargo run --bin clipd
```

What happens:

- SQLite database created or migrated at `%LOCALAPPDATA%\clipmgr\history.db`
- Named pipe server listening at `\\.\pipe\clipmgr`
- Clipboard watcher begins capturing entries every time the clipboard changes

Optional environment tweaks:

```powershell
$env:RUST_LOG = "clipd=debug"
$env:CLIPMGR_MAX_ENTRIES = 5000
```

Stop the daemon with `Ctrl+C` or `Stop-Process -Name clipd`.

## Running the TUI (`clipctl`)

### After Installation

If you used `install.ps1`, press `F12` in any PowerShell window to launch `clipctl`. The installer configures this hotkey automatically.

You can also run it manually:

```powershell
clipctl
```

### Development Mode

Launch the UI in another terminal or Windows Terminal pane:

```powershell
cargo run --bin clipctl
```

Exiting:

- `Enter`/`l` pastes selection and closes the UI
- `q` or `Esc` quits without pasting
- `?` opens the help screen showing all keybindings (press any key to close)

### Manual Windows Terminal Hotkey (Optional)

If you prefer using Windows Terminal's native hotkey system instead of the PowerShell profile F12 binding, add this to your `settings.json`:

```json
{
  "command": {
    "action": "splitPane",
    "split": "right",
    "size": 0.3,
    "commandline": "clipctl"
  },
  "keys": "f12"
}
```

Note: The installer already sets up F12 via PowerShell profile, so this is only needed if you want Windows Terminal-level control.

## Daily Workflows

- Copy from any Windows app; `clipd` captures text, URLs, RTF, and bitmap images automatically.
- Open `clipctl` (`F12`), navigate with `j/k` or arrow keys, preview details in the right pane.
- Press `Enter` or `l` to paste into the focused window.
- Tags help organize snippets: press `t` to add, `T` to remove.
- Press `?` to view the help screen with all available keybindings.

## Visual Features

The TUI includes several visual enhancements to improve usability:

- **Color Themes**: Beautiful color schemes including Nord (default), Dracula, Tokyo Night, and Gruvbox themes with carefully chosen colors for borders, text, and UI elements.
- **Syntax Highlighting**: Automatically detects and highlights code snippets for Rust, Python, JavaScript, Go, C++, Java, SQL, Bash, PHP, and more. Code is highlighted using syntect with a dark theme optimized for terminal viewing.
- **Rich Text Rendering**: Markdown-style formatting support with colored headers (`#`, `##`), bullet points, inline code blocks, and bold text.
- **Colored Icons**: Different colored icons for entry types:
  - 📝 Text (cyan)
  - 🔗 URLs (blue)
  - 🖼️ Images (purple)
  - 📄 RTF/Documents (yellow)
- **Enhanced Preview**: The right pane shows:
  - Entry type and source process
  - Tags with styled backgrounds
  - Timestamp of capture
  - Syntax-highlighted code or formatted text preview
- **Mode-Aware Command Bar**: The bottom command bar shows different prompts with emojis depending on the current mode:
  - 🔍 Search mode
  - 🏷️ Add tag mode
  - 🗑️ Remove tag mode
  - 💾 Export mode
  - 📥 Import mode

## Search & Filtering

1. Press `/` to enter search mode.
2. Type your query; results update in real time across content and tags.
3. Press `Enter` or `Esc` to leave search mode.

Search hints:

- Substring matches are case-insensitive.
- `Enter` on an empty query restores the full list.

## Export & Import

Export current history:

```text
e → clipboard_export.json → Enter
```

Import from a file:

```text
i → backups/snippets.json → Enter
```

Imports deduplicate entries using SHA-256 hashes and log skipped counts.

## Tags & Metadata

- Tags display inline with styled backgrounds in the history list.
- Preview pane shows type icon, source process (e.g., `chrome.exe`), captured timestamp, and content snippet.
- Source tracking uses `GetForegroundWindow` to capture the originating process name.
- Metadata is displayed with styled labels and values for easy scanning.

## Troubleshooting

- **Daemon not running:** `Get-Process clipd` to confirm; restart with `cargo run --bin clipd`.
- **UI cannot connect:** verify pipe name (`$env:CLIPMGR_PIPE`) matches or set a custom path on both processes.
- **Search returns nothing:** ensure history exists, clear filters by typing `/` then pressing `Enter`.
- **Import reports zero entries:** duplicates already in the database; check logs for skipped counts.
- **Clipboard misses entries:** protected UWP apps may block listeners; watcher falls back to polling and surfaces warnings in logs.

## Uninstall & Cleanup

Stop the daemon and remove installed files:

```powershell
# Stop the daemon
Stop-Process -Name clipd -ErrorAction SilentlyContinue

# Remove installed binaries
Remove-Item "$env:LOCALAPPDATA\Programs\rusty-clipboard" -Recurse -Force

# Remove data files
Remove-Item "$env:LOCALAPPDATA\clipmgr\history.db"
Remove-Item "$env:APPDATA\clipmgr\config.toml" -ErrorAction SilentlyContinue

# Clean up environment variables
[Environment]::SetEnvironmentVariable('RUSTY_CLIPBOARD_HOME', $null, 'User')
```

To remove the PowerShell profile integration, edit your PowerShell profile (`$PROFILE`) and delete the section between the markers:

```powershell
# >>> rusty-clipboard integration (managed)
# ... (everything between these markers)
# <<< rusty-clipboard integration (managed)
```

Or run this command to clean it automatically:

```powershell
$profileContent = Get-Content $PROFILE -Raw
$cleaned = $profileContent -replace '(?s)# >>> rusty-clipboard integration \(managed\).*?# <<< rusty-clipboard integration \(managed\)\r?\n?', ''
Set-Content -Path $PROFILE -Value $cleaned.TrimEnd()
```


