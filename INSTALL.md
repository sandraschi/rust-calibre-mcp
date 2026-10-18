# Installing `rust-calibre-mcp` & Calibre ArtCraft Desktop

## Prerequisites

- **Rust 1.85+** (Edition 2024) via `rustup`
- **Windows 10/11 x64**, **macOS 13+**, or **Linux x64**
- Zero external SQLite DLLs required (`rusqlite 0.40` compiles SQLite with `bundled` + `fts5`)

## Build & Install from Source

```powershell
# Clone or enter workspace
cd rust-calibre-mcp

# Build release binaries (L2 CLI/MCP + L3 egui desktop app)
cargo build --workspace --release

# Install both binaries into ~/.cargo/bin
cargo install --path crates/calibre-mcp
cargo install --path crates/calibre-app
```

## Configure in Claude Desktop / Cursor / Windsurf / Antigravity

Add `rust-calibre-mcp` to your MCP client configuration (`claude_desktop_config.json` or `.cursor/mcp.json`):

```json
{
  "mcpServers": {
    "calibre-mcp": {
      "command": "rust-calibre-mcp",
      "args": ["mcp-stdio"]
    }
  }
}
```

## Library Auto-Discovery

On startup, `rust-calibre-mcp` and `calibre-app` automatically discover all local Calibre libraries without manual configuration:

1. `CALIBRE_LIBRARY_PATH` or `CALIBRE_BASE_PATH` environment variable
2. `%APPDATA%\calibre\gui.json` (`library_usage_stats` + `library_path`)
3. `%APPDATA%\calibre\global.py.json` (`library_path`)
4. Sibling directories of any discovered library root (containing `metadata.db`)
5. Deterministic 6-book `Demo-Library` fallback when run on CI or headless machines without Calibre installed

## Verify Installation

```powershell
# 1. Headless L3 smoke check
cargo run -q -p calibre-app -- --smoke

# 2. Live library discovery
cargo run -q -p rust-calibre-mcp -- libraries

# 3. Launch the native egui 0.37 desktop application
cargo run -q -p calibre-app
```
