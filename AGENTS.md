# Agent Handoff Guide

This document helps you switch between different AI agents working on this project while maintaining context and continuity.

## Project Overview

**rusty-clipboard** is a terminal-first clipboard manager for Windows 11 with two main components:
- `clipd`: Background daemon that captures clipboard updates, stores history in SQLite, and serves requests via named pipes
- `clipctl`: Terminal UI client (ratatui) for searching, filtering, and pasting clipboard items with Vim-inspired keybindings

## Current Agent Assignments

### Agent 1: Project Setup & Bootstrap
**Status**: In Progress  
**Focus**: Setting up the project structure, workspace configuration, and initial scaffolding

**Current Tasks**:
- [ ] Workspace setup (Cargo workspace with `clipd` and `clipctl` crates)
- [ ] `clipd` skeleton (Tokio loop, Windows clipboard listener, SQLite persistence, IPC server)
- [ ] `clipctl` skeleton (ratatui TUI, IPC client, navigation/search handlers)
- [ ] Documentation and configuration files

**Key Files**:
- `.cursor/plans/rust-37ce4031.plan.md` - Bootstrap plan
- `Cargo.toml` - Workspace configuration
- `clipd/` - Daemon crate
- `clipctl/` - TUI client crate

### Agent 2: Documentation & Context Management
**Status**: Active  
**Focus**: Creating this handoff document and maintaining project context

**Current Tasks**:
- [x] Create AGENTS.md for agent handoff
- [ ] Maintain project context documentation

## Handoff Instructions

### When Switching Agents

1. **Read this file first** - Understand current agent assignments and project state
2. **Check `.cursor/plans/`** - Review any active plans or task lists
3. **Review recent changes** - Check git status or recent file modifications
4. **Use Context 7** - If available, load context 7 for this project to maintain continuity

### Context 7 Usage

When starting a new session or switching agents:
- **Enable Context 7** in Cursor settings if not already active
- Context 7 helps maintain project-wide understanding across sessions
- Reference this document in your initial prompt: "I'm continuing work on rusty-clipboard. See AGENTS.md for context."

### Quick Context Summary

**Project Structure**:
```
rusty-clipboard/
├── Cargo.toml          # Workspace root
├── clipd/              # Background daemon crate
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── clipboard.rs
│       ├── db.rs
│       ├── ipc.rs
│       └── model.rs
├── clipctl/            # TUI client crate
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── app.rs
│       ├── ui.rs
│       ├── ipc.rs
│       └── paste.rs
├── docs/
│   ├── Architecture.md
│   ├── Design.md
│   ├── Quick_FAQs.md
│   └── Usage.md
├── README.md
└── AGENTS.md
```

**Key Technologies**:
- Rust (latest stable)
- Tokio (async runtime)
- ratatui (TUI framework)
- SQLite (clipboard history storage)
- Windows named pipes (IPC)
- Windows clipboard APIs

**Development Commands**:
```powershell
# Run daemon
cargo run --bin clipd

# Run TUI client
cargo run --bin clipctl
```

## Agent Communication Protocol

### Starting Work
1. Announce which agent you are (or if you're a new agent)
2. State your focus area
3. Review relevant files before making changes
4. Update this document if taking on new tasks

### Completing Work
1. Commit changes using the git commit MCP tool (see Git Operations below)
2. Update task status in this document
3. Note any important decisions or changes
4. Document any blockers or dependencies for next agent

### Git Operations

**IMPORTANT**: Always use the **git commit MCP tool** for all git commit and push operations.

- **Never use terminal commands** like `git commit` or `git push` directly
- **Always use the MCP tool** available in Cursor for committing changes
- The MCP tool helps maintain consistent commit messages and proper git workflow
- When completing work or making significant changes, commit using the MCP tool before switching agents

### Asking for Help
- Reference specific files and line numbers
- Include error messages or unexpected behavior
- Note what you've already tried

## Project Status

**Current Phase**: Bootstrap/Setup  
**Last Updated**: 2025-11-10

**Completed**:
- ✅ Rust installation verified
- ✅ Project structure exists
- ✅ Basic crate scaffolding in place

**In Progress**:
- 🔄 Project setup and bootstrap (Agent 1)

**Blocked/Waiting**:
- None currently

## Documentation Policy

- Keep documentation limited to these markdown files unless the project owner explicitly requests new ones:
  - `README.md` (root overview)
  - `AGENTS.md` (this handoff guide)
  - `docs/Architecture.md`
  - `docs/Design.md`
  - `docs/Usage.md`
  - `docs/Quick_FAQs.md`
- Update the relevant existing file instead of creating new markdown documents.
- If context does not fit any current document, coordinate with the project owner before adding new files.

## Notes for Future Agents

- Rust toolchain is installed and verified (rustc 1.91.0, cargo 1.91.0)
- Project uses a Cargo workspace with two binaries
- Windows-specific implementation (clipboard APIs, named pipes)
- Terminal-first design with Vim-inspired keybindings planned
- See `.cursor/plans/rust-37ce4031.plan.md` for detailed bootstrap plan

---

**Remember**: Always update this document when taking on tasks or completing work to help the next agent understand the current state!

