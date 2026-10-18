# `calibre-core` (L1 Pure-Rust Domain Engine)

`calibre-core` is the L1 pure-Rust library crate for `rust-calibre-mcp`. It implements all 23 canonical commands (`ALL_COMMANDS`), SQLite read-only connection pooling (`SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_NO_MUTEX` + `5000 ms` `busy_timeout`), multi-library discovery across `%APPDATA%\calibre\gui.json` and `global.py.json`, full-text search across `full-text-search.db` (`books_text` + `books_fts` FTS5), cover image loading, book excerpt pagination, reading statistics, unread priority queues, federated multi-library search, write-safety fenced metadata updates, fleet crossconnect (`speech-mcp` & `aiwatcher`), and local/cloud LLM chat (`gemma4:12b` default).

## Modules

- `calibre_db.rs`: Contention-free SQLite reader & analyzer for `metadata.db` and `full-text-search.db` + deterministic 6-book `Demo-Library` fallback.
- `import.rs`: Multi-library discovery (`%APPDATA%\calibre\gui.json`, `global.py.json`, sibling library scan).
- `bridge.rs`: Fleet crossconnect (`speech-mcp` `:10909`, `aiwatcher` `:10946`).
- `llm.rs`: Local (`ollama`, `lmstudio`) and cloud (`openai`, `anthropic`) LLM chat client.
- `creds.rs`: Explicit > environment > OS keyring credential resolution.
- `error.rs`: Typed `DeviceError` (`Timeout`, `Protocol`, `Offline`, `InvalidInput`, `Auth`, `Unsupported`) — zero `.unwrap()` or `panic!`.
