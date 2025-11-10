# Design Notes

This document captures the reasoning behind key architectural and UX decisions, the testing strategy, and areas of ongoing attention for Rusty Clipboard.

## Product Pillars

- **Terminal-first ergonomics:** Everything is optimized for keyboard-heavy workflows with predictable, Vim-inspired interactions.
- **Reliability over novelty:** Clipboard capture must be trustworthy; any missed event erodes confidence.
- **User trust:** Treat clipboard history as sensitive data—provide transparency, local control, and clear escape hatches.

## System Design Decisions

### Clipboard Monitoring

- **Listener + polling fallback:** Prefer `AddClipboardFormatListener` for real-time events, but automatically fall back to `GetClipboardSequenceNumber` polling (250 ms) when sandboxed apps block listeners.
- **UTF-16 normalization:** All text normalizes to UTF-8 to keep downstream processing consistent.
- **SHA-256 dedupe:** Adjacent duplicate suppression (in-memory) plus database-level unique hash ensures O(1) duplicate checks.

### Persistence

- **SQLite (WAL mode):** Balances low write latency with concurrent reads from `clipctl`. WAL files are pruned via periodic `VACUUM` and entry-limits.
- **Schema flexibility:** `tags` stored as JSON text to avoid join overhead while keeping room for future structured queries.
- **Retention policy:** `CLIPMGR_MAX_ENTRIES` enforces bounded history; pruning runs synchronously after inserts to keep disk usage predictable.

### IPC Protocol

- **Named pipes on Windows:** Low-overhead, privilege-friendly transport without firewall prompts.
- **Framed JSON messages:** 32-bit length prefix with JSON payloads simplifies debugging and stays backward compatible as fields grow.
- **Command taxonomy:** `List`, `Search`, `AddTag`, `RemoveTag`, `Export`, `Import`, and `Paste` cover current needs while avoiding version churn.

### UI Architecture

- **Modal input system:** Modes (`Normal`, `Search`, `AddTag`, `RemoveTag`, `Export`, `Import`, `Help`) make complex workflows feel familiar to Vim users.
- **Rich metadata view:** Icons for entry type (📝, 🔗, 🖼️, 📄), tags inline, source process, timestamps, and content preview provide context without leaving the terminal.
- **Borrow checker friendly rendering:** Data is extracted before drawing to avoid mutable aliasing inside `ratatui` rendering closures.

## Testing & Quality

- **Unit tests:** Focus on dedupe hashing, SQLite migrations, IPC serialization/deserialization, and regex/search helpers.
- **Integration tests:** Exercise named-pipe handshake, concurrent UI sessions, and WAL durability under write pressure.
- **Manual QA:** Resize handling, large clipboard payloads, Unicode edge cases, simulated “clipboard storms,” and paste accuracy in different foreground apps.
- **CI expectations:** `cargo fmt`, `cargo clippy --all-targets --all-features`, targeted unit tests, schema verification scripts.

## Risk & Mitigation Highlights

| Risk | Impact | Mitigation |
|------|--------|------------|
| SendInput targets wrong window | Silent paste failure | Confirm foreground window before inject; expose OSC 52/stdout fallback with user feedback. |
| Listener blocked by sandboxed apps | Missed clipboard entries | Polling fallback + log banner when polling is active. |
| SQLite WAL growth during long sessions | Unbounded disk usage | Retention cap (`CLIPMGR_MAX_ENTRIES`) and scheduled vacuum/cleanup. |
| Privacy-sensitive data accumulation | Loss of user trust | Document secure wipe controls, expand configurable ignore patterns, prioritize privacy toggle. |
| TUI crash or resize panic | Loss of control surface | Defensive rendering (no `unwrap` in hot paths), auto-reconnect to daemon on restart. |
| Large binary payloads (images) | Memory spikes, slow persistence | Configurable size caps, deferred image persistence for heavy workloads, future thumbnail pipeline. |

## Extensibility Roadmap

- **Transformers:** Async formatter pipeline for JSON prettify, whitespace trim, future plugins. Interfaces are scoped so transformers can run server-side without UI changes.
- **Sync & sharing:** Potential OneDrive/DevDrive integration by syncing SQLite WAL. Conflict resolution would rely on content hashing.
- **Plugin marketplace:** Signed transformers or UI extensions remain viable thanks to the strict `clipd`/`clipctl` separation.
- **Cross-platform ports:** Architecture leaves room for platform-specific backends; named-pipe abstraction can swap to domain sockets or TCP on other OSes.

## UX Enhancements in Place

- **Discoverability:** `?` toggles an inline help overlay listing every keybinding.
- **Real-time feedback:** Search updates as the user types; command bar mirrors current mode and input buffer.
- **Safe interactions:** `Esc` cancels any mode, avoiding destructive actions.
- **Batch operations (planned):** Future work includes entry pinning, multi-select, and quick filters without breaking current flows.

## Deployment Notes

- **Automated installer:** `install.ps1` handles the complete setup process—builds release binaries, installs to `%LOCALAPPDATA%\Programs\rusty-clipboard`, configures PATH and environment variables, updates PowerShell profile for auto-start and F12 hotkey, and launches the daemon. The integration is idempotent and includes clean removal of previous installations.
- **PowerShell profile integration:** Daemon auto-starts when PowerShell sessions initialize; F12 keybinding uses PSReadLine for seamless TUI invocation without disrupting the command line.
- **Logging:** Set `RUST_LOG` ranges (`clipd=debug,clipctl=debug`) for verbose diagnostics; defaults keep noise low.
- **Binary footprint:** Two executables remain under ~6 MB even with bundled SQLite, staying Microsoft Defender friendly.

## Success Criteria

- Clipboard capture remains lossless across typical workflows.
- Users can discover and execute advanced operations (tags, import/export) entirely through the TUI.
- Latency stays sub-100 ms for capture-to-display and paste-to-foreground interactions.
- Documentation and diagnostics keep onboarding friction low for future contributors.


