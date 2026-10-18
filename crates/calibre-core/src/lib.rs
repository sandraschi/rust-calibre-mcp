//! L1 Calibre core: SQLite `metadata.db` & `full-text-search.db` reader,
//! zero-config `import.rs` from `../calibre-mcp`, fleet `bridge.rs`, and `llm.rs` (`gemma4:12b`).
//!
//! UI runtimes (`calibre-app` egui), transports (`rust-calibre-mcp` stdio), and CLIs
//! route all operations through [`dispatch`] with typed [`Command`] variants.
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unimplemented)]
#![deny(clippy::todo)]

pub mod bridge;
pub mod calibre_db;
pub mod creds;
pub mod error;
pub mod import;
pub mod llm;

use calibre_db::BookQueryParams;
use error::DeviceError;
use serde_json::{json, Value};

pub const CMD_STATUS: &str = "status";
pub const CMD_LIBRARIES_LIST: &str = "calibre.libraries_list";
pub const CMD_LIBRARY_STATS: &str = "calibre.library_stats";
pub const CMD_QUERY_BOOKS: &str = "calibre.query_books";
pub const CMD_BOOK_GET: &str = "calibre.book_get";
pub const CMD_SEARCH_FULLTEXT: &str = "calibre.search_fulltext";
pub const CMD_BOOKCASE_CATALOG: &str = "calibre.bookcase_catalog";
pub const CMD_AUTHORS_LIST: &str = "calibre.authors_list";
pub const CMD_TAGS_LIST: &str = "calibre.tags_list";
pub const CMD_SERIES_ANALYSIS: &str = "calibre.series_analysis";
pub const CMD_FIND_DUPLICATES: &str = "calibre.find_duplicates";
pub const CMD_LIBRARY_HEALTH: &str = "calibre.library_health";
pub const CMD_BRIDGE_EVENT: &str = "bridge.event";
pub const CMD_BRIDGE_SPEAK: &str = "bridge.speak";
pub const CMD_BRIDGE_WEBHOOK: &str = "bridge.webhook";
pub const CMD_LLM_CHAT: &str = "llm.chat";
pub const CMD_LLM_MODELS: &str = "llm.models";

/// Bounded command bus for `rust-calibre-mcp`.
#[derive(Debug, Clone)]
pub enum Command {
    Status,
    LibrariesList {
        include_counts: bool,
    },
    LibraryStats {
        library: Option<String>,
    },
    QueryBooks(BookQueryParams),
    BookGet {
        library: Option<String>,
        book_id: i64,
    },
    SearchFulltext {
        library: Option<String>,
        query: String,
        limit: usize,
    },
    BookcaseCatalog {
        library: Option<String>,
        limit: usize,
    },
    AuthorsList {
        library: Option<String>,
        query: Option<String>,
        limit: usize,
    },
    TagsList {
        library: Option<String>,
        query: Option<String>,
        limit: usize,
    },
    SeriesAnalysis {
        library: Option<String>,
        query: Option<String>,
        limit: usize,
    },
    FindDuplicates {
        library: Option<String>,
        limit: usize,
    },
    LibraryHealth {
        library: Option<String>,
    },
    BridgeEvent {
        kind: String,
        device: String,
        payload: Value,
        url: Option<String>,
    },
    BridgeSpeak {
        text: String,
        voice: Option<String>,
        url: Option<String>,
    },
    BridgeWebhook {
        url: String,
        payload: Value,
    },
    LlmChat {
        provider: String,
        model: Option<String>,
        prompt: String,
        system: Option<String>,
        context: Option<Value>,
        base_url: Option<String>,
        api_key: Option<String>,
    },
    LlmModels {
        provider: String,
        base_url: Option<String>,
        api_key: Option<String>,
    },
}

fn encode(cmd_name: &'static str, result: Result<Value, DeviceError>) -> Value {
    match result {
        Ok(v) => v,
        Err(e) => json!({
            "error": e.to_string(),
            "command": cmd_name,
        }),
    }
}

/// Single infallible entry point for L2 (CLI + MCP stdio) and L3 (`egui` desktop app).
pub async fn dispatch(cmd: Command) -> Value {
    match cmd {
        Command::Status => {
            let summary = import::import_summary();
            json!({
                "status": "ok",
                "server": "rust-calibre-mcp",
                "version": env!("CARGO_PKG_VERSION"),
                "commands": [
                    CMD_STATUS,
                    CMD_LIBRARIES_LIST,
                    CMD_LIBRARY_STATS,
                    CMD_QUERY_BOOKS,
                    CMD_BOOK_GET,
                    CMD_SEARCH_FULLTEXT,
                    CMD_BOOKCASE_CATALOG,
                    CMD_AUTHORS_LIST,
                    CMD_TAGS_LIST,
                    CMD_SERIES_ANALYSIS,
                    CMD_FIND_DUPLICATES,
                    CMD_LIBRARY_HEALTH,
                    CMD_BRIDGE_EVENT,
                    CMD_BRIDGE_SPEAK,
                    CMD_BRIDGE_WEBHOOK,
                    CMD_LLM_CHAT,
                    CMD_LLM_MODELS,
                ],
                "discovery": summary,
            })
        }
        Command::LibrariesList { include_counts } => encode(
            CMD_LIBRARIES_LIST,
            tokio::task::spawn_blocking(move || calibre_db::libraries_list(include_counts))
                .await
                .map_err(|e| DeviceError::Protocol(format!("join error: {e}")))
                .and_then(|r| r),
        ),
        Command::LibraryStats { library } => encode(
            CMD_LIBRARY_STATS,
            tokio::task::spawn_blocking(move || calibre_db::library_stats(library.as_deref()))
                .await
                .map_err(|e| DeviceError::Protocol(format!("join error: {e}")))
                .and_then(|r| r),
        ),
        Command::QueryBooks(params) => encode(
            CMD_QUERY_BOOKS,
            tokio::task::spawn_blocking(move || calibre_db::query_books(&params))
                .await
                .map_err(|e| DeviceError::Protocol(format!("join error: {e}")))
                .and_then(|r| r),
        ),
        Command::BookGet { library, book_id } => encode(
            CMD_BOOK_GET,
            tokio::task::spawn_blocking(move || calibre_db::book_get(library.as_deref(), book_id))
                .await
                .map_err(|e| DeviceError::Protocol(format!("join error: {e}")))
                .and_then(|r| r),
        ),
        Command::SearchFulltext {
            library,
            query,
            limit,
        } => encode(
            CMD_SEARCH_FULLTEXT,
            tokio::task::spawn_blocking(move || {
                calibre_db::search_fulltext(library.as_deref(), &query, limit)
            })
            .await
            .map_err(|e| DeviceError::Protocol(format!("join error: {e}")))
            .and_then(|r| r),
        ),
        Command::BookcaseCatalog { library, limit } => encode(
            CMD_BOOKCASE_CATALOG,
            tokio::task::spawn_blocking(move || {
                calibre_db::bookcase_catalog(library.as_deref(), limit)
            })
            .await
            .map_err(|e| DeviceError::Protocol(format!("join error: {e}")))
            .and_then(|r| r),
        ),
        Command::AuthorsList {
            library,
            query,
            limit,
        } => encode(
            CMD_AUTHORS_LIST,
            tokio::task::spawn_blocking(move || {
                calibre_db::authors_list(library.as_deref(), query.as_deref(), limit)
            })
            .await
            .map_err(|e| DeviceError::Protocol(format!("join error: {e}")))
            .and_then(|r| r),
        ),
        Command::TagsList {
            library,
            query,
            limit,
        } => encode(
            CMD_TAGS_LIST,
            tokio::task::spawn_blocking(move || {
                calibre_db::tags_list(library.as_deref(), query.as_deref(), limit)
            })
            .await
            .map_err(|e| DeviceError::Protocol(format!("join error: {e}")))
            .and_then(|r| r),
        ),
        Command::SeriesAnalysis {
            library,
            query,
            limit,
        } => encode(
            CMD_SERIES_ANALYSIS,
            tokio::task::spawn_blocking(move || {
                calibre_db::series_analysis(library.as_deref(), query.as_deref(), limit)
            })
            .await
            .map_err(|e| DeviceError::Protocol(format!("join error: {e}")))
            .and_then(|r| r),
        ),
        Command::FindDuplicates { library, limit } => encode(
            CMD_FIND_DUPLICATES,
            tokio::task::spawn_blocking(move || {
                calibre_db::find_duplicates(library.as_deref(), limit)
            })
            .await
            .map_err(|e| DeviceError::Protocol(format!("join error: {e}")))
            .and_then(|r| r),
        ),
        Command::LibraryHealth { library } => encode(
            CMD_LIBRARY_HEALTH,
            tokio::task::spawn_blocking(move || calibre_db::library_health(library.as_deref()))
                .await
                .map_err(|e| DeviceError::Protocol(format!("join error: {e}")))
                .and_then(|r| r),
        ),
        Command::BridgeEvent {
            kind,
            device,
            payload,
            url,
        } => encode(
            CMD_BRIDGE_EVENT,
            bridge::aiwatcher_event(
                url,
                &format!("{kind} ({device})"),
                Some(payload.to_string()),
                None,
            )
            .await,
        ),
        Command::BridgeSpeak { text, voice, url } => {
            encode(CMD_BRIDGE_SPEAK, bridge::speak(url, &text, voice).await)
        }
        Command::BridgeWebhook { url, payload } => encode(
            CMD_BRIDGE_WEBHOOK,
            bridge::webhook(&url, "rust-calibre-mcp", Some(payload.to_string())).await,
        ),
        Command::LlmChat {
            provider,
            model,
            prompt,
            system,
            context,
            base_url,
            api_key,
        } => {
            let mut msgs = Vec::new();
            if let Some(sys) = system {
                if !sys.trim().is_empty() {
                    msgs.push(llm::ChatMessage::system(sys));
                }
            }
            if let Some(ctx) = context {
                msgs.push(llm::ChatMessage::system(format!(
                    "Calibre Library Context JSON:\n{ctx}"
                )));
            }
            msgs.push(llm::ChatMessage::user(prompt));
            let resolved_model = model.clone();
            let resolved_provider = provider.clone();
            encode(
                CMD_LLM_CHAT,
                llm::chat(Some(provider), base_url, model, api_key, msgs)
                    .await
                    .map(|reply| {
                        json!({
                            "provider": resolved_provider,
                            "model": resolved_model.unwrap_or_else(|| "gemma4:12b".to_string()),
                            "reply": reply,
                        })
                    }),
            )
        }
        Command::LlmModels {
            provider,
            base_url,
            api_key,
        } => encode(
            CMD_LLM_MODELS,
            llm::models(Some(provider), base_url, api_key)
                .await
                .map(|models| json!({"models": models})),
        ),
    }
}

pub fn is_invalid_input(value: &Value) -> bool {
    value
        .get("error")
        .and_then(Value::as_str)
        .unwrap_or("")
        .contains("invalid input")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn status_lists_all_17_commands() {
        let v = dispatch(Command::Status).await;
        let cmds = v
            .get("commands")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert_eq!(cmds.len(), 17);
    }

    #[tokio::test]
    async fn empty_fulltext_query_returns_typed_error_not_panic() {
        let v = dispatch(Command::SearchFulltext {
            library: Some("demo".to_string()),
            query: "   ".to_string(),
            limit: 10,
        })
        .await;
        assert!(is_invalid_input(&v));
    }

    #[tokio::test]
    async fn demo_query_books_round_trip() {
        let v = dispatch(Command::QueryBooks(BookQueryParams {
            library: Some("demo".to_string()),
            query: Some("Rust".to_string()),
            ..Default::default()
        }))
        .await;
        assert_eq!(v["count"], 1);
    }

    #[test]
    fn dispatch_future_is_send() {
        fn assert_send<T: Send>(_: T) {}
        assert_send(dispatch(Command::Status));
    }
}
