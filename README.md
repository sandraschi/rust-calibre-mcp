# rust-calibre-mcp

Pure-Rust Calibre e-book manager interface, 23-tool Model Context Protocol (MCP) server, and native GPU-rendered desktop dashboard.

## What this wraps

This project interfaces with **[Calibre](https://calibre-ebook.com/)**, the popular open-source e-book manager. It connects directly to local Calibre libraries on disk via fast, read-only SQLite queries. See [docs/WRAPPEE.md](docs/WRAPPEE.md) for Calibre project links and community resources.

## What You Can Do

- **Search & Filter**: Search book titles, authors, genres/tags, series, and formats across multiple libraries simultaneously.
- **Deep Full-Text Search**: Search directly inside book bodies via Calibre's full-text index without external dependencies.
- **Book Inspector**: View cover art, metadata, formats on disk, and paginated text excerpts directly inside the native desktop app.
- **Reading Queue & Analytics**: View unread books ranked by ratings and series order, publication decade charts, and series completion gap analysis.
- **AI Librarian Copilot**: Ask questions about your collection with automatic book synopsis and context injection (local Ollama or cloud models).
- **Fleet Crossconnect**: Speak book synopses aloud via `speech-mcp` or push events to `aiwatcher`.
- **Zero-Corruption Read-Only Engine**: All SQLite queries run in strict read-only mode, safe to use even while the Calibre desktop app is open.

## Install

### 1. Windows Installer (Recommended)
1. Download **`Calibre-Rust-0.4.0-setup.exe`** from the [latest release](https://github.com/sandraschi/rust-calibre-mcp/releases/latest).
2. Run the installer. Windows may show a SmartScreen warning for unsigned binaries: choose **More info > Run anyway**.
3. Open **Calibre (Rust)** from your Start menu.

### 2. For Claude Desktop / Cursor / OpenCode (MCP Server Only)
Add `rust-calibre-mcp` to your MCP client configuration (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "calibre": {
      "command": "C:\\path\\to\\rust-calibre-mcp.exe",
      "args": ["mcp-stdio"]
    }
  }
}
```

For full installation options across Windows, macOS, and Linux, see [INSTALL.md](INSTALL.md).  
Developers: build instructions and test recipes are in [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

## Example Prompts

Once configured in Claude or your AI agent, you can ask natural-language questions:

- *"What science fiction books in my Calibre collection have a 5-star rating?"*
- *"Find any books mentioning 'quantum computing' in the full-text search index and show me excerpts."*
- *"Are there any missing books in my Frank Herbert Dune series?"*
- *"Give me a summary of my unread priority reading list."*

## Documentation

| Document | Description |
|---|---|
| [Installation Guide](INSTALL.md) | Full setup options (installer, portable exe, build from source) |
| [Wrapped Application](docs/WRAPPEE.md) | Calibre project information, documentation, and community links |
| [Architecture](docs/ARCHITECTURE.md) | 3-crate ArtCraft architecture, SQLite read-only safety, and FTS5 bypass |
| [Configuration](docs/CONFIGURATION.md) | Environment variables, discovery priority, and settings |
| [Tool Reference](docs/TOOLS.md) | Complete list of all 23 canonical MCP tools and legacy aliases |
| [Development](docs/DEVELOPMENT.md) | Source build guide, justfile recipes, and testing gates |
| [UI Parity Specification](docs/UI_PARITY.md) | `egui 0.37` desktop dashboard layout and visual standards |
| [Troubleshooting](docs/TROUBLESHOOTING.md) | Common questions, SmartScreen help, and diagnostic steps |

## Requirements

- **Operating System**: Windows 10/11, macOS 13+, or Linux x64
- **Calibre**: Calibre 5.0+ installed locally with at least one library (or run in Demo mode)
- **Claude / Cursor**: Any MCP-compatible AI client

## License

Dual-licensed under MIT OR Apache-2.0.
