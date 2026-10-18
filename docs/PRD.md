# Product Requirements Document (PRD) -- `rust-calibre-mcp`

**Date**: 2026-10-18
**Status**: Wave-0 Reference Pilot (`repconv` Belt-and-Suspenders Standard)
**Legacy Oracle**: `D:\Dev\repos\calibre-mcp` (`master@5abaad6`, untouched read-only)

## 1. Objective

Replace the 209-Python-file `calibre-mcp` stack (FastMCP + FastAPI + SQLAlchemy + LanceDB + Vite React webapp) with a single native Rust workspace (`rust-calibre-mcp`) that starts in `< 15 ms`, consumes `< 25 MB` RAM idle, never mutates or locks Calibre's SQLite files, and provides:

1. **`calibre-core` (L1)**: Typed async `Command` / `dispatch()` core over `rusqlite` (`bundled`) with 23 canonical capabilities.
2. **`rust-calibre-mcp` (L2)**: CLI (`--json`) + `MCP 2024-11-05` stdio server with 23 canonical `calibre.*` / `bridge.*` / `llm.*` tools and legacy portmanteau aliases.
3. **`calibre-app` (L3)**: Native `eframe`/`egui 0.37` desktop UI with full structural clone parity across all 16 pages in 5 grouped sections (`CONTENT`, `DISCOVER`, `MANAGE`, `AI & FLEET`, `SYSTEM`), `Ctrl+Wheel` zoom (`0.5`..`2.0`), dark/light theme persistence, and the **Fleet Standard LLM Settings Card** (provider selection, live model elicitation, Windows Credential Manager OS vaulting via `keyring`, and live connection test).

## 2. Hard Invariants (10-Point Uncrashable Binary Contract)

1. **Zero Panic Surface**: `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo)]` in every crate.
2. **Read-Only SQLite Guard**: `OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX` + `busy_timeout(5000 ms)`.
3. **Custom FTS Tokenizer Bypass**: Direct `books_text` scan in `full-text-search.db` so Calibre's custom `calibre` FTS5 tokenizer never causes a runtime error.
4. **Dual Tool-Dispatcher Aliasing**: Both canonical `calibre.*` tools and all legacy FastMCP 3.2 portmanteau tool names resolve cleanly.
5. **Full Structural Webapp Clone Parity**: The desktop app must be a faithful, fully functional clone of the legacy webapp (every navigational group and subpage present, zero missing views).
6. **Fleet Standard LLM Settings Card**: All rustified apps must provide provider choice (Ollama, LM Studio, OpenAI, Anthropic, Custom), dynamic model elicitation from live endpoints, OS Credential Manager secret storage, and non-streaming connectivity testing.
7. **Glyph Budget**: Only ASCII and `['°', '×', '·', '…']` non-ASCII glyphs permitted in `crates/calibre-app/src/app.rs`, verified by `glyph_source_scan`.
8. **Zero Legacy Mutation**: `D:\Dev\repos\calibre-mcp` remains 100% untouched.
