# rusty-clipboard

Rusty Clipboard is a terminal-first clipboard manager for Windows 11. It pairs a background clipboard capture daemon (`clipd`) with a right-hand terminal UI (`clipctl`) that lets you search, filter, and paste items using Vim-inspired keybindings.

## Crates

- `clipd`: background service that listens for clipboard updates, normalizes content, stores history in SQLite, and serves requests via a named pipe.
- `clipctl`: terminal UI client built with ratatui that displays clipboard history, supports incremental search, syntax highlighting, multiple color themes, and triggers paste actions back into the active terminal.

## Installation

### Prerequisites

- Windows 11 22H2 or later
- Rust toolchain (install with: `winget install --id Rustlang.Rust.MSVC`)

### Quick Install

Run the PowerShell installer to build the binaries, install them into your
user profile, hook up an F12 launcher, and start the background daemon:

```powershell
.\install.ps1
```

The script will:
- Build `clipd` and `clipctl` in release mode
- Copy the executables to `%LOCALAPPDATA%\Programs\rusty-clipboard`
- Set the `RUSTY_CLIPBOARD_HOME` environment variable and update your PATH
- Update your PowerShell profile so F12 launches `clipctl` and `clipd` auto-starts
- Start the daemon immediately

Restart your PowerShell session after the installer finishes so the profile
changes take effect. Then press F12 to launch the clipboard manager UI.

### Building from Source

If you've cloned this repository from GitHub, you can build the project manually:

**Prerequisites:**
- Windows 11 22H2 or later
- Rust toolchain (install with: `winget install --id Rustlang.Rust.MSVC`)

**Build Steps:**

1. Clone the repository:
```powershell
git clone https://github.com/yourusername/rusty-clipboard.git
cd rusty-clipboard
```

2. Build the release binaries:
```powershell
cargo build --release --bin clipd --bin clipctl
```

3. The binaries will be in `target/release/`:
   - `target/release/clipd.exe` - Background daemon
   - `target/release/clipctl.exe` - Terminal UI client

4. (Optional) Run the installer to set up your environment:
```powershell
.\install.ps1 --SkipBuild
```

Or manually copy the binaries to your desired location and add them to your PATH.

**For Development:**
```powershell
# Run in debug mode
cargo run --bin clipd
cargo run --bin clipctl

# Or use the dev script
.\dev.ps1
```

## Usage

After installation:
- Open a new PowerShell window
- Press **F12** to launch `clipctl`
- Copy any text/image—it's automatically captured
- Navigate with `j`/`k`, search with `/`, paste with `Enter`
- Press `?` to view the help screen with all keybindings

**Visual Features:**
- Multiple color themes (Nord, Dracula, Tokyo Night, Gruvbox)
- Automatic syntax highlighting for code snippets (Rust, Python, JavaScript, Go, C++, Java, SQL, Bash, and more)
- Rich text rendering with markdown-style formatting
- Colored icons for different content types (text, URLs, images, documents)
- Enhanced preview pane with metadata and styled tags

See `docs/Usage.md` for detailed workflows and `docs/Quick_FAQs.md` for command reference.

## Developing

### Quick Start (Single Script)

Run both `clipd` and `clipctl` with a single command:

```powershell
.\dev.ps1
```

### Manual Start

Or start each component separately:

```powershell
cargo run --bin clipd
cargo run --bin clipctl
```

## Documentation

- `docs/Architecture.md` — system overview and data flow
- `docs/Design.md` — rationale, UX decisions, testing, and risk summary
- `docs/Usage.md` — install, run, common workflows, and troubleshooting
- `docs/Quick_FAQs.md` — fast reference for commands, keybindings, and config
- Default config template: `config/config.example.toml`
