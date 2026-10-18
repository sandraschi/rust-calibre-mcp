# Configuration Reference — `rust-calibre-mcp`

`rust-calibre-mcp` is zero-config by default. When run without arguments, it automatically discovers local libraries. This reference lists optional environment variables and settings.

## Environment Variables

| Variable | Description | Default |
|---|---|---|
| `CALIBRE_LIBRARY_PATH` | Path to a specific Calibre library directory (containing `metadata.db`). | Discovered via `%APPDATA%\calibre` |
| `CALIBRE_BASE_PATH` | Alternative base directory containing multiple Calibre libraries as subdirectories. | None |
| `CALIBRE_MCP_LEGACY_DIR` | Path to legacy `calibre-mcp` for `.env` or `config.json` import. | `../calibre-mcp` |
| `CALIBRE_SERVER_URL` | Optional Calibre Content Server HTTP URL. | None |
| `CALIBRE_PASSWORD` | Optional Calibre Content Server password (can also be saved to OS Keyring). | None |
| `OLLAMA_BASE_URL` | Base URL for Ollama local LLM chat assistant. | `http://localhost:11434` |
| `SPEECH_MCP_URL` | Webhook URL for voice synthesis via `speech-mcp`. | `http://localhost:10909/speak` |
| `AIWATCHER_URL` | Webhook URL for event push via `aiwatcher`. | `http://localhost:10946/api/events` |

## Discovery Order

Settings are resolved in strict priority order:
1. Explicit CLI arguments (`--library <name-or-path>`) or MCP tool parameters
2. Environment variables (`CALIBRE_LIBRARY_PATH`)
3. OS Calibre configuration (`%APPDATA%\calibre\gui.json` & `global.py.json`)
4. Sibling legacy configuration (`../calibre-mcp/config.json`)
5. Sibling directory scan in the parent folder of any discovered library root
6. Built-in 6-book `Demo-Library` fallback for offline / CI environments
