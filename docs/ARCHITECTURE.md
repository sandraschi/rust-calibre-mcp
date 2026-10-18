# Architecture & Engine Design — `rust-calibre-mcp`

This document details the internal architecture, SQLite read-only guarantees, FTS5 tokenizer bypass, and workspace layering for `rust-calibre-mcp`.

## 1. The 3-Crate ArtCraft Seam

```text
rust-calibre-mcp/
├── Cargo.toml                  # Virtual workspace + #![deny(unwrap_used, expect_used, panic)]
├── justfile                    # check / test / smoke / release recipes
├── AGENTS.md                   # ArtCraft containment harness
└── crates/
    ├── calibre-core/           # L1: Pure domain library, SQLite engine, dispatch bus
    ├── calibre-mcp/            # L2: CLI binary + MCP 2024-11-05 JSON-RPC stdio server
    └── calibre-app/            # L3: Native egui 0.37 desktop application
```

| Layer | Crate | Dependencies | Responsibility |
|---|---|---|---|
| **L1 Core** | [`crates/calibre-core`](../crates/calibre-core/README.md) | `rusqlite`, `serde`, `tokio`, `reqwest` | Pure domain logic. Zero stdout/stderr. All operations mapped to typed `Command` variants and executed via `dispatch(cmd: Command) -> Value`. |
| **L2 CLI + MCP** | [`crates/calibre-mcp`](../crates/calibre-mcp/README.md) | `clap`, `serde_json`, `calibre-core` | CLI subcommands (`--json`) + stdin/stdout JSON-RPC 2.0 loop (`mcp-stdio`). Dual-aliased routing maps legacy portmanteau tool names to `Command` variants. |
| **L3 Desktop GUI** | [`crates/calibre-app`](../crates/calibre-app/README.md) | `eframe`, `egui 0.37`, `calibre-core` | Native GPU-rendered UI. Calls `calibre_core::dispatch` in-process via an async background worker channel (`mpsc`), zero HTTP overhead. |

---

## 2. Zero-Corruption Read-Only SQLite Guarantee

Calibre's desktop application writes directly to `metadata.db` and `full-text-search.db`. To guarantee that `rust-calibre-mcp` never causes lock contention or database corruption:

1. **Read-Only Flags**:
   Every reader connection is opened with:
   ```rust
   let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
   let conn = Connection::open_with_flags(db_path, flags)?;
   let _ = conn.busy_timeout(Duration::from_millis(5000));
   ```
2. **Write-Safety Fence**:
   Mutating operations (`calibre.book_update`) default to `confirm_write = false`. In dry-run mode, the engine returns a preview JSON object without opening SQLite in write mode. Only when `confirm_write = true` is explicitly passed on a non-demo library does the engine execute an atomic transaction.

---

## 3. FTS5 Custom Tokenizer Bypass

Calibre's desktop app registers a proprietary C tokenizer (`calibre remove_diacritics`) on `full-text-search.db:books_fts`. Standard external SQLite clients fail if they query `books_fts` directly.

`rust-calibre-mcp` bypasses this limitation cleanly:
- Queries the underlying content table `full-text-search.db:books_text` directly using SQL snippet extraction (`substr` + `instr`).
- Implements UTF-8 character-safe slicing in `calibre_db.rs:truncate_chars` to prevent panics at multi-byte boundaries.
- Requires zero external C plugins or proprietary Calibre DLLs.

---

## 4. Universal Command Bus (`Command::dispatch`)

All 23 capabilities are defined as variants of `calibre_core::Command`:

```rust
pub enum Command {
    LibrariesList { include_counts: bool },
    LibraryStats { library: Option<String> },
    QueryBooks(BookQueryParams),
    BookGet { library: Option<String>, book_id: i64 },
    BookCover { library: Option<String>, book_id: i64 },
    BookExcerpt { library: Option<String>, book_id: i64, format: Option<String>, offset_chars: usize, max_chars: usize },
    BookUpdate { library: Option<String>, book_id: i64, title: Option<String>, comment_html: Option<String>, confirm_write: bool },
    SearchAllLibraries { query: String, limit_per_library: usize },
    SearchFulltext { library: Option<String>, query: String, limit: usize },
    AuthorsList { library: Option<String>, limit: usize },
    TagsList { library: Option<String>, limit: usize },
    SeriesList { library: Option<String>, limit: usize },
    PublishersList { library: Option<String>, limit: usize },
    LibraryHealth { library: Option<String> },
    Duplicates { library: Option<String> },
    ReadingStats { library: Option<String> },
    UnreadPriority { library: Option<String>, limit: usize },
    ExportJson { library: Option<String>, limit: usize },
    ImportConfig,
    BridgeEvent { kind: String, device: String, payload: Value, url: Option<String> },
    BridgeSpeak { text: String, voice: Option<String>, url: Option<String> },
    LlmChat { provider: String, model: Option<String>, prompt: String, system: Option<String>, context: Option<Value>, base_url: Option<String>, api_key: Option<String> },
    Status,
}
```

Every command is tested for `Send` safety at compile time via `dispatch_future_is_send`.

---

## 5. L3 Native Desktop GUI Architecture (`calibre-app`)

The desktop app in `crates/calibre-app` operates as an immediate-mode GPU application with an asynchronous background Tokio worker:

1. **16-Page Navigation Router (`Page`)**:
   - `CONTENT`: `Page::Libraries`, `Page::Books`, `Page::Authors`, `Page::Series`, `Page::Tags`, `Page::Publishers`.
   - `DISCOVER`: `Page::Fulltext`, `Page::Reading`, `Page::Duplicates`.
   - `MANAGE`: `Page::Health`.
   - `AI & FLEET`: `Page::Chat`, `Page::Crossconnect`.
   - `SYSTEM`: `Page::Logs`, `Page::Settings`, `Page::Help`, `Page::RawJson`.
2. **Zero-UI-Thread-Blocking Worker Channel (`mpsc::Sender<Job>`)**:
   - All SQLite queries, HTTP crossconnect requests, LLM model elicitation, and Windows Credential Manager operations are dispatched via `self.spawn(...)` to the Tokio runtime thread.
   - Completed results are returned as `Job` variants (`Job::Books`, `Job::Authors`, `Job::Series`, `Job::Tags`, `Job::Duplicates`, `Job::Health`, `Job::ReadingStats`, `Job::VaultLoaded`, `Job::LlmModelsLoaded`, etc.) and drained cleanly on every render frame via `self.drain_jobs(&ctx)`.
3. **OS Credential Manager Security (`keyring`)**:
   - Calibre Content Server passwords and Cloud LLM API keys are stored strictly in the Windows Credential Manager under the target `rust-calibre-mcp`.
   - Plain text secrets are never persisted to disk or serialized in config files.
