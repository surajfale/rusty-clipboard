# Architecture

Rusty Clipboard splits clipboard capture and the terminal UI into two focused executables that communicate over Windows named pipes. This keeps the always-on daemon lean while letting the UI restart freely without losing history.

## Guiding Principles

- Latency first: end-to-end interactions should stay under 100 ms, so hot paths rely on async IO and bounded work.
- Process isolation: `clipd` (daemon) and `clipctl` (TUI) fail independently; UI issues never block capture.
- Windows-native: we lean on Win32 APIs (`AddClipboardFormatListener`, `GetClipboardSequenceNumber`, `SendInput`, named pipes) to avoid extra drivers or services.
- Observability without friction: structured logging (`tracing`) and simple JSON IPC payloads keep debugging approachable.

## Component Responsibilities

| Component | Role | Lifecycle |
|-----------|------|-----------|
| `clipd` | Captures clipboard updates, deduplicates entries, persists to SQLite, serves IPC requests. | Starts at login or manually; runs headless until stopped. |
| `clipctl` | Keyboard-driven TUI for browsing, searching, tagging, and pasting clipboard history. | Launch on demand (e.g., `F12` shortcut); exits when user quits or pastes. |

This boundary guarantees that clipboard history remains intact even when the UI restarts.

## Data Flow

```
User copies text/image ─┐
                        ▼
          Windows Clipboard APIs
                        ▼
           Clipboard watcher (clipd)
                        ▼
              Deduplication hash
                        ▼
                SQLite persistence
                        ▲
                        │ Named pipe (JSON)
                        ▼
             clipctl renders + actions
```

## Clipboard Capture

- Primary strategy uses `AddClipboardFormatListener`; a polling fallback watches `GetClipboardSequenceNumber` to tolerate restricted apps.
- Dedicated single-threaded Tokio runtime pumps the Win32 message loop and forwards normalized UTF-8 entries over async channels.
- SHA-256 hashes suppress adjacent duplicates before disk writes.

## Persistence

- SQLite runs in WAL mode to balance <5 ms writes with concurrent readers.
- History lives in `%LOCALAPPDATA%\clipmgr\history.db`, scoped per Windows user profile.
- Schema: `entries(id PRIMARY KEY, created_at INTEGER, kind TEXT, text TEXT, bytes_len INTEGER, hash TEXT UNIQUE, source_process TEXT, tags TEXT)`.
- Pruning enforces `CLIPMGR_MAX_ENTRIES` (default 10 000) after each insert.

## IPC Layer

- Transport: `\\.\pipe\clipmgr` named pipe with 32-bit length-prefixed JSON frames for request/response symmetry.
- JSON keeps payloads human-readable; if serialization becomes the bottleneck we can swap to MessagePack behind the same framing.
- Backoff and retry on the client side mask short-lived daemon restarts.

## TUI Rendering

- `ratatui` + `crossterm` render a three-pane layout: history list, preview pane, command bar.
- `clipctl` polls for input via `spawn_blocking` around `crossterm::event::read`, keeping the async runtime responsive.
- Modal UI mirrors Vim semantics for predictable keyboard-driven workflows.

## Search & Filtering

- Server-side filtering keeps UI logic simple: IPC supports `List`, `Search`, and tag mutation commands.
- Initial matching performs substring queries; roadmap includes fuzzy ranking and regex without changing the protocol.
- Highlighting and inline match indicators are handled purely in the UI layer.

## Operational Considerations

- **Clipboard hook gaps:** Some UWP apps block listeners; polling fallback and warning banners mitigate missed events.
- **SendInput focus issues:** UI verifies foreground window before injecting keystrokes and can fall back to OSC 52 / stdout when necessary.
- **WAL growth:** Periodic vacuum plus entry pruning keeps the database bounded during long sessions.
- **Security posture:** History stays local and unencrypted by default; privacy mode and secure wipe remain high-priority backlog items.

## Extensibility Hints

- Transformer pipeline hooks allow formatters (JSON prettify, trimming) to run inline in future releases.
- Named-pipe framing already versioned; adding new request types or optional fields remains backwards-compatible.
- Sync, cloud backup, or plugin marketplaces can sit on top without rewriting the daemon core thanks to the strict module boundaries.


