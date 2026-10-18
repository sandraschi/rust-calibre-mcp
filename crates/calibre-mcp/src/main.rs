//! L2 CLI + Dual-Aliased `MCP 2024-11-05` stdio server for `rust-calibre-mcp`.
//!
//! Routes all commands through [`calibre_core::dispatch`] and supports both
//! ArtCraft dotted/underscored tool names (`calibre.query_books` / `calibre_query_books`)
//! and legacy Python `calibre-mcp` portmanteau tool names (`manage_libraries`,
//! `query_books`, `manage_books`, `search_fulltext`, `manage_authors`, `manage_tags`,
//! `get_series_analysis`, `find_duplicate_books`, `analyze_library_health`, `inspect_contents`,
//! `reading_statistics`, `unread_priority_list`, etc.).
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unimplemented)]
#![deny(clippy::todo)]

use calibre_core::calibre_db::BookQueryParams;
use calibre_core::{
    dispatch, Command, CMD_AUTHORS_LIST, CMD_BOOKCASE_CATALOG, CMD_BOOK_COVER, CMD_BOOK_EXCERPT,
    CMD_BOOK_GET, CMD_BOOK_UPDATE, CMD_BRIDGE_EVENT, CMD_BRIDGE_SPEAK, CMD_BRIDGE_WEBHOOK,
    CMD_FIND_DUPLICATES, CMD_LIBRARIES_LIST, CMD_LIBRARY_HEALTH, CMD_LIBRARY_STATS, CMD_LLM_CHAT,
    CMD_LLM_MODELS, CMD_QUERY_BOOKS, CMD_READING_STATS, CMD_SEARCH_ALL_LIBRARIES,
    CMD_SEARCH_FULLTEXT, CMD_SERIES_ANALYSIS, CMD_STATUS, CMD_TAGS_LIST, CMD_UNREAD_PRIORITY,
};
use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

/// Install a belt-and-suspenders panic hook that logs to `%LOCALAPPDATA%\rust-calibre-mcp\crash.log`
/// and `stderr` (never `stdout`, preserving MCP JSON-RPC wire purity).
fn install_forensic_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let msg = format!("[{ts}] PANIC in rust-calibre-mcp: {info}\n");
        let _ = io::stderr().write_all(msg.as_bytes());
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let dir = PathBuf::from(local).join("rust-calibre-mcp");
            let _ = std::fs::create_dir_all(&dir);
            let log_file = dir.join("crash.log");
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)
            {
                let _ = f.write_all(msg.as_bytes());
            }
        }
    }));
}

#[derive(Parser, Debug)]
#[command(
    name = "rust-calibre-mcp",
    version,
    about = "Native Rust Calibre Library CLI & MCP 2024-11-05 Server"
)]
struct Cli {
    /// Emit compact single-line JSON instead of pretty-printed JSON.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    cmd: Sub,
}

#[derive(Subcommand, Debug)]
enum Sub {
    /// Server status and discovered Calibre libraries summary.
    Status,
    /// List all discovered Calibre libraries (`metadata.db` + `full-text-search.db`).
    Libraries {
        #[arg(long, default_value_t = true)]
        counts: bool,
    },
    /// Show detailed statistics for a Calibre library.
    LibraryStats {
        #[arg(long)]
        library: Option<String>,
    },
    /// Query and filter books across title, author, tag, series, and format.
    QueryBooks {
        #[arg(long)]
        library: Option<String>,
        #[arg(long)]
        query: Option<String>,
        #[arg(long)]
        author: Option<String>,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long)]
        series: Option<String>,
        #[arg(long)]
        format: Option<String>,
        #[arg(long, default_value_t = 25)]
        limit: usize,
        #[arg(long, default_value_t = 0)]
        offset: usize,
    },
    /// Get full book metadata, comments/synopsis, identifiers, and format file paths by ID.
    BookGet {
        #[arg(long)]
        book_id: i64,
        #[arg(long)]
        library: Option<String>,
    },
    /// Inspect cover metadata and byte size for a book (`cover.jpg`).
    BookCover {
        #[arg(long)]
        book_id: i64,
        #[arg(long)]
        library: Option<String>,
    },
    /// Read a character-safe excerpt of a book's text from `full-text-search.db:books_text`.
    #[command(visible_alias = "excerpt")]
    BookExcerpt {
        #[arg(long)]
        book_id: i64,
        #[arg(long)]
        library: Option<String>,
        #[arg(long)]
        format: Option<String>,
        #[arg(long, default_value_t = 0)]
        offset: usize,
        #[arg(long, default_value_t = 1200)]
        max_chars: usize,
    },
    /// Update book title or comment/synopsis (write-safety fenced: dry-run unless `--confirm-write`).
    BookUpdate {
        #[arg(long)]
        book_id: i64,
        #[arg(long)]
        library: Option<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        comment: Option<String>,
        #[arg(long, default_value_t = false)]
        confirm_write: bool,
    },
    /// Search inside book full-text content (`full-text-search.db`) with snippet extraction.
    #[command(visible_alias = "fts")]
    SearchFulltext {
        #[arg(long)]
        query: String,
        #[arg(long)]
        library: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Search across all discovered Calibre libraries on disk simultaneously.
    #[command(visible_alias = "search-all")]
    SearchAllLibraries {
        #[arg(long)]
        query: String,
        #[arg(long, default_value_t = 10)]
        limit_per_library: usize,
    },
    /// Group books by primary tag shelf (`bookcase_catalog`).
    Bookcase {
        #[arg(long)]
        library: Option<String>,
        #[arg(long, default_value_t = 60)]
        limit: usize,
    },
    /// List authors with book counts.
    Authors {
        #[arg(long)]
        library: Option<String>,
        #[arg(long)]
        query: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// List tags with book counts and unused tag count.
    Tags {
        #[arg(long)]
        library: Option<String>,
        #[arg(long)]
        query: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Series and canon coverage analysis (identifies missing volume indices).
    Series {
        #[arg(long)]
        library: Option<String>,
        #[arg(long)]
        query: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Detect duplicate book titles in a library.
    Duplicates {
        #[arg(long)]
        library: Option<String>,
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
    /// Audit library metadata health (missing covers, missing comments, duplicates).
    Health {
        #[arg(long)]
        library: Option<String>,
    },
    /// Language, publisher, publication decade, and rating distribution statistics.
    ReadingStats {
        #[arg(long)]
        library: Option<String>,
    },
    /// Ranked unread/priority reading queue (top-rated and series-opener books with EPUB/PDF).
    UnreadPriority {
        #[arg(long)]
        library: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Speak text aloud via fleet `speech-mcp` (`:10909`).
    Speak {
        #[arg(long)]
        text: String,
        #[arg(long)]
        voice: Option<String>,
        #[arg(long)]
        url: Option<String>,
    },
    /// Chat with local Ollama (`gemma4:12b` default) or OpenAI/Anthropic about your library.
    Chat {
        #[arg(long)]
        prompt: String,
        #[arg(long, default_value = "ollama")]
        provider: String,
        #[arg(long)]
        model: Option<String>,
        #[arg(long)]
        system: Option<String>,
    },
    /// List models available on the configured LLM provider.
    Models {
        #[arg(long, default_value = "ollama")]
        provider: String,
    },
    /// Run the `MCP 2024-11-05` JSON-RPC stdio server.
    #[command(visible_alias = "mcp-stdio")]
    Mcp,
}

fn str_arg(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
}

fn usize_arg(args: &Value, key: &str, default: usize) -> usize {
    args.get(key)
        .and_then(Value::as_u64)
        .and_then(|n| usize::try_from(n).ok())
        .unwrap_or(default)
}

fn i64_arg(args: &Value, key: &str) -> Option<i64> {
    args.get(key).and_then(Value::as_i64).or_else(|| {
        args.get(key)
            .and_then(Value::as_str)
            .and_then(|s| s.trim().parse::<i64>().ok())
    })
}

/// Complete MCP tool catalog (23 canonical ArtCraft tools).
pub fn tools_list() -> Vec<Value> {
    vec![
        json!({
            "name": CMD_STATUS,
            "description": "Return rust-calibre-mcp server status, command catalog, and auto-discovered Calibre libraries.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": CMD_LIBRARIES_LIST,
            "description": "List all discovered Calibre libraries (metadata.db and full-text-search.db) with book counts and sizes.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "include_counts": { "type": "boolean", "default": true }
                }
            }
        }),
        json!({
            "name": CMD_LIBRARY_STATS,
            "description": "Get comprehensive statistics (total books, authors, tags, series, publishers, formats, cover/comment coverage) for a Calibre library.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "library": { "type": "string", "description": "Library name or path (default: Calibre-Bibliothek, or 'demo')" }
                }
            }
        }),
        json!({
            "name": CMD_QUERY_BOOKS,
            "description": "Search and filter books in a Calibre library by keyword query, author, tag, series, or format.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "library": { "type": "string" },
                    "query": { "type": "string" },
                    "author": { "type": "string" },
                    "tag": { "type": "string" },
                    "series": { "type": "string" },
                    "format": { "type": "string" },
                    "limit": { "type": "integer", "default": 50 },
                    "offset": { "type": "integer", "default": 0 }
                }
            }
        }),
        json!({
            "name": CMD_BOOK_GET,
            "description": "Get full metadata, plain/HTML comments, identifiers (ISBN/DOI), cover path, and format files for a single book ID.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "library": { "type": "string" },
                    "book_id": { "type": "integer" }
                },
                "required": ["book_id"]
            }
        }),
        json!({
            "name": CMD_BOOK_COVER,
            "description": "Inspect cover.jpg metadata and byte size for a book ID.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "library": { "type": "string" },
                    "book_id": { "type": "integer" }
                },
                "required": ["book_id"]
            }
        }),
        json!({
            "name": CMD_BOOK_EXCERPT,
            "description": "Read a character-safe excerpt of a book's text from full-text-search.db:books_text (or synopsis fallback).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "library": { "type": "string" },
                    "book_id": { "type": "integer" },
                    "format": { "type": "string" },
                    "offset_chars": { "type": "integer", "default": 0 },
                    "max_chars": { "type": "integer", "default": 1200 }
                },
                "required": ["book_id"]
            }
        }),
        json!({
            "name": CMD_BOOK_UPDATE,
            "description": "Update a book's title or comment/synopsis with write-safety fence (dry-run preview unless confirm_write=true).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "library": { "type": "string" },
                    "book_id": { "type": "integer" },
                    "new_title": { "type": "string" },
                    "new_comment_html": { "type": "string" },
                    "confirm_write": { "type": "boolean", "default": false }
                },
                "required": ["book_id"]
            }
        }),
        json!({
            "name": CMD_SEARCH_FULLTEXT,
            "description": "Search inside full book text using Calibre's full-text-search.db (books_text) with context snippets.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "library": { "type": "string" },
                    "query": { "type": "string" },
                    "limit": { "type": "integer", "default": 25 }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": CMD_SEARCH_ALL_LIBRARIES,
            "description": "Federated search across all discovered Calibre libraries on disk.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "limit_per_library": { "type": "integer", "default": 10 }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": CMD_BOOKCASE_CATALOG,
            "description": "Group books into visual tag shelves (bookcase catalog).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "library": { "type": "string" },
                    "limit": { "type": "integer", "default": 60 }
                }
            }
        }),
        json!({
            "name": CMD_AUTHORS_LIST,
            "description": "List authors in a Calibre library ordered by book count.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "library": { "type": "string" },
                    "query": { "type": "string" },
                    "limit": { "type": "integer", "default": 50 }
                }
            }
        }),
        json!({
            "name": CMD_TAGS_LIST,
            "description": "List tags in a Calibre library with book counts and unused tag count.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "library": { "type": "string" },
                    "query": { "type": "string" },
                    "limit": { "type": "integer", "default": 50 }
                }
            }
        }),
        json!({
            "name": CMD_SERIES_ANALYSIS,
            "description": "Analyze book series and canon coverage (identifies missing volume numbers in each series).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "library": { "type": "string" },
                    "query": { "type": "string" },
                    "limit": { "type": "integer", "default": 50 }
                }
            }
        }),
        json!({
            "name": CMD_FIND_DUPLICATES,
            "description": "Find duplicate books sharing normalized titles inside a Calibre library.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "library": { "type": "string" },
                    "limit": { "type": "integer", "default": 25 }
                }
            }
        }),
        json!({
            "name": CMD_LIBRARY_HEALTH,
            "description": "Audit library metadata health (missing covers, missing comments/descriptions, duplicate title groups).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "library": { "type": "string" }
                }
            }
        }),
        json!({
            "name": CMD_READING_STATS,
            "description": "Get language distribution, top publishers, publication decades, and rating breakdown for a Calibre library.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "library": { "type": "string" }
                }
            }
        }),
        json!({
            "name": CMD_UNREAD_PRIORITY,
            "description": "Get a ranked priority reading queue (top-rated and series-opener books with readable formats).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "library": { "type": "string" },
                    "limit": { "type": "integer", "default": 20 }
                }
            }
        }),
        json!({
            "name": CMD_BRIDGE_EVENT,
            "description": "Push a structured library event to aiwatcher (:10946).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "kind": { "type": "string" },
                    "device": { "type": "string" },
                    "payload": { "type": "object" },
                    "url": { "type": "string" }
                },
                "required": ["kind", "device"]
            }
        }),
        json!({
            "name": CMD_BRIDGE_SPEAK,
            "description": "Speak book synopses or notifications aloud via speech-mcp (:10909).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "text": { "type": "string" },
                    "voice": { "type": "string" },
                    "url": { "type": "string" }
                },
                "required": ["text"]
            }
        }),
        json!({
            "name": CMD_BRIDGE_WEBHOOK,
            "description": "Post JSON payload to an automation webhook.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "url": { "type": "string" },
                    "payload": { "type": "object" }
                },
                "required": ["url"]
            }
        }),
        json!({
            "name": CMD_LLM_CHAT,
            "description": "Chat with local Ollama (gemma4:12b default) or cloud LLM with optional Calibre library context.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "provider": { "type": "string", "default": "ollama" },
                    "model": { "type": "string" },
                    "prompt": { "type": "string" },
                    "system": { "type": "string" },
                    "context": { "type": "object" }
                },
                "required": ["prompt"]
            }
        }),
        json!({
            "name": CMD_LLM_MODELS,
            "description": "List models available on the selected LLM provider.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "provider": { "type": "string", "default": "ollama" }
                }
            }
        }),
    ]
}

/// Dual-aliased tool router: accepts canonical ArtCraft names (`calibre.query_books`),
/// underscored names (`calibre_query_books`), and all 29 legacy Python `calibre-mcp` tool names.
pub async fn call_tool(name: &str, args: &Value) -> Result<Value, String> {
    let lib = str_arg(args, "library").or_else(|| str_arg(args, "library_name"));
    let op = str_arg(args, "operation")
        .unwrap_or_default()
        .to_lowercase();

    let cmd = match name {
        CMD_STATUS
        | "calibre.status"
        | "calibre_status"
        | "manage_system"
        | "help_tool"
        | "show_api_docs"
        | "show_libraries_prefab_card" => Command::Status,

        CMD_LIBRARIES_LIST
        | "calibre_libraries_list"
        | "list_libraries_helper"
        | "library_discovery" => {
            let include_counts = args
                .get("include_counts")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            Command::LibrariesList { include_counts }
        }

        "manage_libraries" | "manage_library_operations" => match op.as_str() {
            "stats" | "statistics" => Command::LibraryStats { library: lib },
            "search" | "cross_search" => {
                let query = str_arg(args, "query")
                    .ok_or_else(|| "missing required parameter 'query'".to_string())?;
                Command::SearchAllLibraries {
                    query,
                    limit_per_library: usize_arg(args, "limit", 10),
                }
            }
            _ => Command::LibrariesList {
                include_counts: true,
            },
        },

        CMD_LIBRARY_STATS | "calibre_library_stats" | "get_library_stats_helper" => {
            Command::LibraryStats { library: lib }
        }

        CMD_QUERY_BOOKS
        | "calibre_query_books"
        | "query_books"
        | "search_books_helper"
        | "list_books_helper"
        | "calibre_metadata_search" => Command::QueryBooks(BookQueryParams {
            library: lib,
            query: str_arg(args, "query")
                .or_else(|| str_arg(args, "search_text"))
                .or_else(|| str_arg(args, "title")),
            author: str_arg(args, "author"),
            tag: str_arg(args, "tag"),
            series: str_arg(args, "series"),
            format: str_arg(args, "format"),
            limit: Some(usize_arg(args, "limit", 50)),
            offset: Some(usize_arg(args, "offset", 0)),
        }),

        CMD_SEARCH_ALL_LIBRARIES
        | "calibre_search_all_libraries"
        | "cross_library_search_helper" => {
            let query = str_arg(args, "query")
                .or_else(|| str_arg(args, "search_text"))
                .ok_or_else(|| "missing required parameter 'query'".to_string())?;
            Command::SearchAllLibraries {
                query,
                limit_per_library: usize_arg(args, "limit_per_library", 10),
            }
        }

        "get_books_by_author_helper" => Command::QueryBooks(BookQueryParams {
            library: lib,
            author: str_arg(args, "author").or_else(|| str_arg(args, "query")),
            limit: Some(usize_arg(args, "limit", 50)),
            ..Default::default()
        }),

        "get_books_by_series_helper" => Command::QueryBooks(BookQueryParams {
            library: lib,
            series: str_arg(args, "series").or_else(|| str_arg(args, "query")),
            limit: Some(usize_arg(args, "limit", 50)),
            ..Default::default()
        }),

        CMD_BOOK_COVER | "calibre_book_cover" => {
            if let Some(book_id) = i64_arg(args, "book_id").or_else(|| i64_arg(args, "id")) {
                Command::BookCover {
                    library: lib,
                    book_id,
                }
            } else {
                return Err("missing required integer parameter 'book_id'".to_string());
            }
        }

        CMD_BOOK_EXCERPT | "calibre_book_excerpt" | "inspect_contents" | "manage_viewer" => {
            if let Some(book_id) = i64_arg(args, "book_id").or_else(|| i64_arg(args, "id")) {
                Command::BookExcerpt {
                    library: lib,
                    book_id,
                    format: str_arg(args, "format"),
                    offset_chars: usize_arg(args, "offset_chars", 0),
                    max_chars: usize_arg(args, "max_chars", 1200),
                }
            } else {
                return Err("missing required integer parameter 'book_id'".to_string());
            }
        }

        CMD_BOOK_UPDATE | "calibre_book_update" => {
            if let Some(book_id) = i64_arg(args, "book_id").or_else(|| i64_arg(args, "id")) {
                let confirm_write = args
                    .get("confirm_write")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                Command::BookUpdate {
                    library: lib,
                    book_id,
                    new_title: str_arg(args, "new_title").or_else(|| str_arg(args, "title")),
                    new_comment_html: str_arg(args, "new_comment_html")
                        .or_else(|| str_arg(args, "comment"))
                        .or_else(|| str_arg(args, "description")),
                    confirm_write,
                }
            } else {
                return Err("missing required integer parameter 'book_id'".to_string());
            }
        }

        "manage_comments"
        | "manage_descriptions"
        | "manage_metadata"
        | "manage_extended_metadata" => {
            if let Some(book_id) = i64_arg(args, "book_id").or_else(|| i64_arg(args, "id")) {
                let new_title = str_arg(args, "new_title").or_else(|| str_arg(args, "title"));
                let new_comment = str_arg(args, "new_comment_html")
                    .or_else(|| str_arg(args, "comment"))
                    .or_else(|| str_arg(args, "text"))
                    .or_else(|| str_arg(args, "description"));
                if op == "update" || op == "set" || new_title.is_some() || new_comment.is_some() {
                    let confirm_write = args
                        .get("confirm_write")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    Command::BookUpdate {
                        library: lib,
                        book_id,
                        new_title,
                        new_comment_html: new_comment,
                        confirm_write,
                    }
                } else {
                    Command::BookGet {
                        library: lib,
                        book_id,
                    }
                }
            } else {
                return Err("missing required integer parameter 'book_id'".to_string());
            }
        }

        CMD_BOOK_GET
        | "calibre_book_get"
        | "get_book_helper"
        | "manage_files"
        | "export_books"
        | "show_book_prefab_card" => {
            if let Some(book_id) = i64_arg(args, "book_id").or_else(|| i64_arg(args, "id")) {
                Command::BookGet {
                    library: lib,
                    book_id,
                }
            } else {
                return Err("missing required integer parameter 'book_id'".to_string());
            }
        }

        "manage_books" => {
            if op == "list" || op == "search" {
                Command::QueryBooks(BookQueryParams {
                    library: lib,
                    query: str_arg(args, "query"),
                    limit: Some(usize_arg(args, "limit", 50)),
                    offset: Some(usize_arg(args, "offset", 0)),
                    ..Default::default()
                })
            } else if let Some(book_id) = i64_arg(args, "book_id").or_else(|| i64_arg(args, "id")) {
                Command::BookGet {
                    library: lib,
                    book_id,
                }
            } else {
                Command::QueryBooks(BookQueryParams {
                    library: lib,
                    limit: Some(usize_arg(args, "limit", 25)),
                    ..Default::default()
                })
            }
        }

        CMD_SEARCH_FULLTEXT
        | "calibre_search_fulltext"
        | "search_fulltext"
        | "rag_retrieve"
        | "calibre_rag"
        | "rag_index_build"
        | "calibre_metadata_index_build" => {
            let query = str_arg(args, "query")
                .or_else(|| str_arg(args, "search_text"))
                .ok_or_else(|| "missing required parameter 'query'".to_string())?;
            Command::SearchFulltext {
                library: lib,
                query,
                limit: usize_arg(args, "limit", 25),
            }
        }

        CMD_BOOKCASE_CATALOG | "calibre_bookcase_catalog" | "bookcase_catalog" => {
            Command::BookcaseCatalog {
                library: lib,
                limit: usize_arg(args, "limit", 60),
            }
        }

        CMD_AUTHORS_LIST | "calibre_authors_list" | "manage_authors" | "manage_publishers" => {
            Command::AuthorsList {
                library: lib,
                query: str_arg(args, "query").or_else(|| str_arg(args, "name")),
                limit: usize_arg(args, "limit", 50),
            }
        }

        CMD_TAGS_LIST
        | "calibre_tags_list"
        | "manage_tags"
        | "get_tag_statistics"
        | "list_tags"
        | "get_simple_tag_statistics" => Command::TagsList {
            library: lib,
            query: str_arg(args, "query").or_else(|| str_arg(args, "name")),
            limit: usize_arg(args, "limit", 50),
        },

        CMD_SERIES_ANALYSIS
        | "calibre_series_analysis"
        | "get_series_analysis"
        | "canon_coverage" => Command::SeriesAnalysis {
            library: lib,
            query: str_arg(args, "query").or_else(|| str_arg(args, "series")),
            limit: usize_arg(args, "limit", 50),
        },

        CMD_FIND_DUPLICATES | "calibre_find_duplicates" | "find_duplicate_books" => {
            Command::FindDuplicates {
                library: lib,
                limit: usize_arg(args, "limit", 25),
            }
        }

        CMD_READING_STATS | "calibre_reading_stats" | "reading_statistics" => {
            Command::ReadingStats { library: lib }
        }

        CMD_UNREAD_PRIORITY | "calibre_unread_priority" | "unread_priority_list" => {
            Command::UnreadPriority {
                library: lib,
                limit: usize_arg(args, "limit", 20),
            }
        }

        CMD_LIBRARY_HEALTH
        | "calibre_library_health"
        | "analyze_library_health"
        | "analyze_library"
        | "manage_analysis"
        | "calibre_metadata_export_json"
        | "manage_bulk_operations"
        | "manage_content_sync"
        | "manage_smart_collections" => Command::LibraryHealth { library: lib },

        CMD_BRIDGE_EVENT | "bridge_event" => {
            let kind = str_arg(args, "kind").ok_or_else(|| "missing 'kind'".to_string())?;
            let device = str_arg(args, "device").ok_or_else(|| "missing 'device'".to_string())?;
            let payload = args.get("payload").cloned().unwrap_or_else(|| json!({}));
            Command::BridgeEvent {
                kind,
                device,
                payload,
                url: str_arg(args, "url"),
            }
        }

        CMD_BRIDGE_SPEAK | "bridge_speak" => {
            let text = str_arg(args, "text").ok_or_else(|| "missing 'text'".to_string())?;
            Command::BridgeSpeak {
                text,
                voice: str_arg(args, "voice"),
                url: str_arg(args, "url"),
            }
        }

        CMD_BRIDGE_WEBHOOK | "bridge_webhook" => {
            let url = str_arg(args, "url").ok_or_else(|| "missing 'url'".to_string())?;
            let payload = args.get("payload").cloned().unwrap_or_else(|| json!({}));
            Command::BridgeWebhook { url, payload }
        }

        CMD_LLM_CHAT
        | "llm_chat"
        | "agentic_library_workflow"
        | "agentic_calibre_workflow"
        | "conversational_calibre_assistant"
        | "intelligent_library_processing" => {
            let prompt = str_arg(args, "prompt")
                .or_else(|| str_arg(args, "goal"))
                .or_else(|| str_arg(args, "query"))
                .ok_or_else(|| "missing required parameter 'prompt'".to_string())?;
            Command::LlmChat {
                provider: str_arg(args, "provider").unwrap_or_else(|| "ollama".to_string()),
                model: str_arg(args, "model"),
                prompt,
                system: str_arg(args, "system"),
                context: args.get("context").cloned(),
                base_url: str_arg(args, "base_url"),
                api_key: str_arg(args, "api_key"),
            }
        }

        CMD_LLM_MODELS | "llm_models" => Command::LlmModels {
            provider: str_arg(args, "provider").unwrap_or_else(|| "ollama".to_string()),
            base_url: str_arg(args, "base_url"),
            api_key: str_arg(args, "api_key"),
        },

        other => return Err(format!("unknown tool '{other}'")),
    };

    Ok(dispatch(cmd).await)
}

/// Handle a single JSON-RPC `2.0` request line, returning `Some(response)` for requests
/// and `None` for notifications.
pub async fn handle_mcp_line(line: &str) -> Option<Value> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    let req: Value = match serde_json::from_str(trimmed) {
        Ok(v) => v,
        Err(e) => {
            return Some(json!({
                "jsonrpc": "2.0",
                "id": Value::Null,
                "error": { "code": -32700, "message": format!("Parse error: {e}") }
            }));
        }
    };

    let id = req.get("id").cloned();
    let method = req.get("method").and_then(Value::as_str).unwrap_or("");
    if id.is_none() || method.starts_with("notifications/") {
        return None;
    }
    let id_val = id.unwrap_or(Value::Null);

    match method {
        "initialize" => Some(json!({
            "jsonrpc": "2.0",
            "id": id_val,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "rust-calibre-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        })),
        "ping" => Some(json!({
            "jsonrpc": "2.0",
            "id": id_val,
            "result": {}
        })),
        "tools/list" => Some(json!({
            "jsonrpc": "2.0",
            "id": id_val,
            "result": { "tools": tools_list() }
        })),
        "tools/call" => {
            let params = req.get("params").cloned().unwrap_or_else(|| json!({}));
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            match call_tool(name, &args).await {
                Ok(val) => {
                    let is_err = val.get("error").is_some();
                    let text =
                        serde_json::to_string_pretty(&val).unwrap_or_else(|_| "{}".to_string());
                    Some(json!({
                        "jsonrpc": "2.0",
                        "id": id_val,
                        "result": {
                            "content": [{ "type": "text", "text": text }],
                            "isError": is_err
                        }
                    }))
                }
                Err(msg) => Some(json!({
                    "jsonrpc": "2.0",
                    "id": id_val,
                    "result": {
                        "content": [{ "type": "text", "text": msg }],
                        "isError": true
                    }
                })),
            }
        }
        other => Some(json!({
            "jsonrpc": "2.0",
            "id": id_val,
            "error": { "code": -32601, "message": format!("Method not found: {other}") }
        })),
    }
}

async fn run_mcp_stdio() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if let Some(resp) = handle_mcp_line(&line).await {
            if let Ok(serialized) = serde_json::to_string(&resp) {
                stdout.write_all(serialized.as_bytes())?;
                stdout.write_all(b"\n")?;
                stdout.flush()?;
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    install_forensic_panic_hook();
    let cli = Cli::parse();

    let cmd = match cli.cmd {
        Sub::Mcp => {
            let _ = run_mcp_stdio().await;
            return;
        }
        Sub::Status => Command::Status,
        Sub::Libraries { counts } => Command::LibrariesList {
            include_counts: counts,
        },
        Sub::LibraryStats { library } => Command::LibraryStats { library },
        Sub::QueryBooks {
            library,
            query,
            author,
            tag,
            series,
            format,
            limit,
            offset,
        } => Command::QueryBooks(BookQueryParams {
            library,
            query,
            author,
            tag,
            series,
            format,
            limit: Some(limit),
            offset: Some(offset),
        }),
        Sub::BookGet { book_id, library } => Command::BookGet { library, book_id },
        Sub::BookCover { book_id, library } => Command::BookCover { library, book_id },
        Sub::BookExcerpt {
            book_id,
            library,
            format,
            offset,
            max_chars,
        } => Command::BookExcerpt {
            library,
            book_id,
            format,
            offset_chars: offset,
            max_chars,
        },
        Sub::BookUpdate {
            book_id,
            library,
            title,
            comment,
            confirm_write,
        } => Command::BookUpdate {
            library,
            book_id,
            new_title: title,
            new_comment_html: comment,
            confirm_write,
        },
        Sub::SearchFulltext {
            query,
            library,
            limit,
        } => Command::SearchFulltext {
            library,
            query,
            limit,
        },
        Sub::SearchAllLibraries {
            query,
            limit_per_library,
        } => Command::SearchAllLibraries {
            query,
            limit_per_library,
        },
        Sub::Bookcase { library, limit } => Command::BookcaseCatalog { library, limit },
        Sub::Authors {
            library,
            query,
            limit,
        } => Command::AuthorsList {
            library,
            query,
            limit,
        },
        Sub::Tags {
            library,
            query,
            limit,
        } => Command::TagsList {
            library,
            query,
            limit,
        },
        Sub::Series {
            library,
            query,
            limit,
        } => Command::SeriesAnalysis {
            library,
            query,
            limit,
        },
        Sub::Duplicates { library, limit } => Command::FindDuplicates { library, limit },
        Sub::Health { library } => Command::LibraryHealth { library },
        Sub::ReadingStats { library } => Command::ReadingStats { library },
        Sub::UnreadPriority { library, limit } => Command::UnreadPriority { library, limit },
        Sub::Speak { text, voice, url } => Command::BridgeSpeak { text, voice, url },
        Sub::Chat {
            prompt,
            provider,
            model,
            system,
        } => Command::LlmChat {
            provider,
            model,
            prompt,
            system,
            context: None,
            base_url: None,
            api_key: None,
        },
        Sub::Models { provider } => Command::LlmModels {
            provider,
            base_url: None,
            api_key: None,
        },
    };

    let out = dispatch(cmd).await;
    let formatted = if cli.json {
        serde_json::to_string(&out)
    } else {
        serde_json::to_string_pretty(&out)
    };
    if let Ok(text) = formatted {
        let _ = writeln!(io::stdout(), "{text}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_lists_23_tools() {
        let tools = tools_list();
        assert_eq!(tools.len(), 23);
    }

    #[tokio::test]
    async fn unknown_tool_becomes_error_content_not_panic() {
        let res = call_tool("nonexistent_calibre_tool", &json!({})).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn legacy_portmanteau_aliases_route_cleanly() {
        let res = call_tool(
            "manage_libraries",
            &json!({"operation": "stats", "library": "demo"}),
        )
        .await;
        assert!(res.is_ok());
        if let Ok(v) = res {
            assert_eq!(v["library_name"], "Demo-Library");
        }

        let q = call_tool("query_books", &json!({"library": "demo", "query": "Dune"})).await;
        assert!(q.is_ok());
        if let Ok(v) = q {
            assert_eq!(v["count"], 2);
        }

        let ex = call_tool(
            "inspect_contents",
            &json!({"library": "demo", "book_id": 1}),
        )
        .await;
        assert!(ex.is_ok());

        let rs = call_tool("reading_statistics", &json!({"library": "demo"})).await;
        assert!(rs.is_ok());
    }

    #[tokio::test]
    async fn missing_required_book_id_is_rejected() {
        let res = call_tool("calibre.book_get", &json!({"library": "demo"})).await;
        assert!(res.is_err());
    }
}
