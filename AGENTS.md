# `rust-calibre-mcp` -- Agent Rules & Containment Contract

**Purpose:** Pure-Rust ArtCraft replacement for `calibre-mcp`. Queries Calibre SQLite libraries (`metadata.db` and `full-text-search.db`) in `< 2 ms` via bundled `rusqlite` (`SQLITE_OPEN_READ_ONLY`), exposes a `clap` CLI (`--json`), a dual-aliased `MCP 2024-11-05` stdio server, and a native `eframe`/`egui` GPU desktop browser (`calibre-app`).

## Hard Containment Rules

1. **Zero `unwrap` / `expect` / `panic` / `todo` / `unimplemented`:** Enforced at workspace level via `[workspace.lints.clippy]`.
2. **Strict L1 -> L2/L3 Layering:**
   - `crates/calibre-core` (L1): SQLite queries, zero-config `import.rs` from `../calibre-mcp`, fleet `bridge.rs`, and `llm.rs` (`gemma4:12b` default).
   - `crates/calibre-mcp` (L2): `clap` CLI + `MCP 2024-11-05` stdio server supporting both `calibre.*` and legacy portmanteau tool names (`manage_libraries`, `query_books`, `manage_books`, `search_fulltext`, etc.).
   - `crates/calibre-app` (L3): Pure-Rust `egui` desktop app (`Libraries`, `Books`, `Analytics`, `Crossconnect`, `Chat`, `Settings`, `Help`, `Raw JSON`).
3. **Read-Only SQLite Contention Safety:** Never lock or mutate external `metadata.db` or `full-text-search.db` during read commands; always open with `SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_NO_MUTEX` + `busy_timeout(5000 ms)`.
4. **Legacy Repo Read-Only Fence:** Never modify `D:\Dev\repos\calibre-mcp`.
