//! Contention-free SQLite reader for Calibre `metadata.db` and `full-text-search.db`,
//! plus deterministic `Demo` library fallback for CI and `--smoke` runs.

use crate::error::DeviceError;
use crate::import::{discover_libraries, DiscoveredLibrary};
use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Parameters for searching/filtering books in a Calibre library.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BookQueryParams {
    pub library: Option<String>,
    pub query: Option<String>,
    pub author: Option<String>,
    pub tag: Option<String>,
    pub series: Option<String>,
    pub format: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Summary record of a book returned by list/search queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookSummary {
    pub id: i64,
    pub title: String,
    pub authors: Vec<String>,
    pub series: Option<String>,
    pub series_index: f64,
    pub tags: Vec<String>,
    pub formats: Vec<String>,
    pub rating: Option<i64>,
    pub pubdate: Option<String>,
    pub timestamp: Option<String>,
    pub has_cover: bool,
    pub cover_path: Option<String>,
    pub library_name: String,
    pub path: String,
    pub summary_snippet: Option<String>,
}

/// UTF-8 safe character truncation (never slices inside a multi-byte codepoint).
pub fn truncate_chars(input: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (idx, ch) in input.chars().enumerate() {
        if idx >= max_chars {
            out.push('…');
            break;
        }
        out.push(ch);
    }
    out
}

/// Strip simple HTML tags from Calibre `comments.text` for clean one-line snippets.
pub fn strip_html_tags(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut in_tag = false;
    for ch in raw.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Open a Calibre SQLite file strictly read-only with a 5-second busy timeout.
fn open_readonly_sqlite(db_path: &Path) -> Result<Connection, DeviceError> {
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn = Connection::open_with_flags(db_path, flags)
        .map_err(|e| DeviceError::Offline(format!("cannot open {}: {e}", db_path.display())))?;
    let _ = conn.busy_timeout(Duration::from_millis(5000));
    Ok(conn)
}

#[derive(Debug, Clone)]
enum ResolvedLibrary {
    Demo,
    Live(DiscoveredLibrary),
}

fn resolve_target_library(requested: Option<&str>) -> Result<ResolvedLibrary, DeviceError> {
    if let Some(req) = requested {
        let trimmed = req.trim();
        if trimmed.eq_ignore_ascii_case("demo")
            || trimmed.eq_ignore_ascii_case("mock")
            || trimmed.eq_ignore_ascii_case("stub")
        {
            return Ok(ResolvedLibrary::Demo);
        }
        let direct_path = PathBuf::from(trimmed);
        let direct_meta = if direct_path.ends_with("metadata.db") {
            direct_path.clone()
        } else {
            direct_path.join("metadata.db")
        };
        if let Ok(stat) = std::fs::metadata(&direct_meta) {
            if stat.is_file() && stat.len() > 0 {
                let dir = direct_meta
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| direct_path.clone());
                let fts = dir.join("full-text-search.db");
                let has_fts = fts.is_file();
                let name = dir
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "CustomLibrary".to_string());
                return Ok(ResolvedLibrary::Live(DiscoveredLibrary {
                    name,
                    path: dir.to_string_lossy().into_owned(),
                    metadata_db: direct_meta.to_string_lossy().into_owned(),
                    fts_db: if has_fts {
                        Some(fts.to_string_lossy().into_owned())
                    } else {
                        None
                    },
                    metadata_size_bytes: stat.len(),
                    has_fts_db: has_fts,
                    is_default: true,
                    source: "explicit_path".to_string(),
                }));
            }
        }

        let discovered = discover_libraries();
        let needle = trimmed.to_lowercase();
        if let Some(found) = discovered
            .iter()
            .find(|l| l.name.to_lowercase() == needle || l.path.to_lowercase() == needle)
            .or_else(|| {
                discovered
                    .iter()
                    .find(|l| l.name.to_lowercase().contains(&needle))
            })
        {
            return Ok(ResolvedLibrary::Live(found.clone()));
        }
        return Err(DeviceError::InvalidInput(format!(
            "Calibre library '{trimmed}' not found (discovered: {})",
            discovered
                .iter()
                .map(|l| l.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }

    let discovered = discover_libraries();
    if let Some(def) = discovered
        .iter()
        .find(|l| l.is_default)
        .or_else(|| discovered.first())
    {
        Ok(ResolvedLibrary::Live(def.clone()))
    } else {
        Ok(ResolvedLibrary::Demo)
    }
}

fn demo_books() -> Vec<BookSummary> {
    vec![
        BookSummary {
            id: 1,
            title: "Dune".to_string(),
            authors: vec!["Frank Herbert".to_string()],
            series: Some("Dune Chronicles".to_string()),
            series_index: 1.0,
            tags: vec!["Science Fiction".to_string(), "Classic".to_string(), "Ecology".to_string()],
            formats: vec!["EPUB".to_string(), "PDF".to_string()],
            rating: Some(10),
            pubdate: Some("1965-08-01".to_string()),
            timestamp: Some("2026-01-10".to_string()),
            has_cover: true,
            cover_path: None,
            library_name: "Demo-Library".to_string(),
            path: "Frank Herbert/Dune (1)".to_string(),
            summary_snippet: Some("Set on the desert planet Arrakis, Dune is the story of the boy Paul Atreides, heir to a noble family tasked with ruling an inhospitable world where the spice melange grows.".to_string()),
        },
        BookSummary {
            id: 2,
            title: "Dune Messiah".to_string(),
            authors: vec!["Frank Herbert".to_string()],
            series: Some("Dune Chronicles".to_string()),
            series_index: 2.0,
            tags: vec!["Science Fiction".to_string(), "Classic".to_string()],
            formats: vec!["EPUB".to_string()],
            rating: Some(8),
            pubdate: Some("1969-10-15".to_string()),
            timestamp: Some("2026-01-12".to_string()),
            has_cover: true,
            cover_path: None,
            library_name: "Demo-Library".to_string(),
            path: "Frank Herbert/Dune Messiah (2)".to_string(),
            summary_snippet: Some("Paul Atreides rules as Emperor Muad'Dib while conspiracies gather across the Imperium.".to_string()),
        },
        BookSummary {
            id: 3,
            title: "The Rust Programming Language".to_string(),
            authors: vec!["Steve Klabnik".to_string(), "Carol Nichols".to_string()],
            series: None,
            series_index: 1.0,
            tags: vec!["Programming".to_string(), "Rust".to_string(), "Systems".to_string()],
            formats: vec!["EPUB".to_string(), "PDF".to_string()],
            rating: Some(10),
            pubdate: Some("2023-02-28".to_string()),
            timestamp: Some("2026-02-01".to_string()),
            has_cover: true,
            cover_path: None,
            library_name: "Demo-Library".to_string(),
            path: "Steve Klabnik/The Rust Programming Language (3)".to_string(),
            summary_snippet: Some("The official guide to Rust ownership, borrowing, lifetimes, fearless concurrency, and zero-cost abstractions.".to_string()),
        },
        BookSummary {
            id: 4,
            title: "Neuromancer".to_string(),
            authors: vec!["William Gibson".to_string()],
            series: Some("Sprawl Trilogy".to_string()),
            series_index: 1.0,
            tags: vec!["Science Fiction".to_string(), "Cyberpunk".to_string()],
            formats: vec!["EPUB".to_string(), "MOBI".to_string()],
            rating: Some(9),
            pubdate: Some("1984-07-01".to_string()),
            timestamp: Some("2026-02-15".to_string()),
            has_cover: true,
            cover_path: None,
            library_name: "Demo-Library".to_string(),
            path: "William Gibson/Neuromancer (4)".to_string(),
            summary_snippet: Some("Case was the sharpest data-thief in the matrix until his nervous system was crippled; now a mysterious employer offers a last run against an orbital AI.".to_string()),
        },
        BookSummary {
            id: 5,
            title: "Designing Data-Intensive Applications".to_string(),
            authors: vec!["Martin Kleppmann".to_string()],
            series: None,
            series_index: 1.0,
            tags: vec!["Programming".to_string(), "Distributed Systems".to_string(), "Databases".to_string()],
            formats: vec!["PDF".to_string(), "EPUB".to_string()],
            rating: Some(10),
            pubdate: Some("2017-03-16".to_string()),
            timestamp: Some("2026-03-01".to_string()),
            has_cover: true,
            cover_path: None,
            library_name: "Demo-Library".to_string(),
            path: "Martin Kleppmann/Designing Data-Intensive Applications (5)".to_string(),
            summary_snippet: Some("Deep dive into storage engines, B-trees, LSM-trees, replication, partitioning, consensus, and batch/stream processing.".to_string()),
        },
        BookSummary {
            id: 6,
            title: "Gödel, Escher, Bach: An Eternal Golden Braid".to_string(),
            authors: vec!["Douglas R. Hofstadter".to_string()],
            series: None,
            series_index: 1.0,
            tags: vec!["Mathematics".to_string(), "Cognitive Science".to_string(), "Logic".to_string()],
            formats: vec!["PDF".to_string()],
            rating: Some(10),
            pubdate: Some("1979-04-09".to_string()),
            timestamp: Some("2026-03-10".to_string()),
            has_cover: false,
            cover_path: None,
            library_name: "Demo-Library".to_string(),
            path: "Douglas R. Hofstadter/Godel, Escher, Bach (6)".to_string(),
            summary_snippet: Some("A metaphorical fugue on minds and machines in the spirit of Lewis Carroll, exploring strange loops and formal systems.".to_string()),
        },
    ]
}

fn split_pipe_list(raw: Option<String>) -> Vec<String> {
    raw.unwrap_or_default()
        .split('|')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect()
}

/// 1. List all discovered Calibre libraries with fast book counts.
pub fn libraries_list(include_counts: bool) -> Result<Value, DeviceError> {
    let libs = discover_libraries();
    if libs.is_empty() {
        return Ok(json!({
            "mode": "demo",
            "total_libraries": 1,
            "default_library": "Demo-Library",
            "libraries": [{
                "name": "Demo-Library",
                "path": "memory://demo",
                "metadata_db": "memory://demo/metadata.db",
                "metadata_size_mb": 0.42,
                "has_fts_db": true,
                "is_default": true,
                "book_count": 6,
                "source": "demo"
            }]
        }));
    }

    let mut out = Vec::with_capacity(libs.len());
    let mut total_books_across_all: i64 = 0;

    for lib in &libs {
        let mut book_count: Option<i64> = None;
        if include_counts {
            if let Ok(conn) = open_readonly_sqlite(Path::new(&lib.metadata_db)) {
                if let Ok(c) =
                    conn.query_row("SELECT COUNT(*) FROM books", [], |r| r.get::<_, i64>(0))
                {
                    book_count = Some(c);
                    total_books_across_all = total_books_across_all.saturating_add(c);
                }
            }
        }
        let size_mb = (lib.metadata_size_bytes as f64) / (1024.0 * 1024.0);
        out.push(json!({
            "name": lib.name,
            "path": lib.path,
            "metadata_db": lib.metadata_db,
            "fts_db": lib.fts_db,
            "metadata_size_mb": (size_mb * 100.0).round() / 100.0,
            "has_fts_db": lib.has_fts_db,
            "is_default": lib.is_default,
            "book_count": book_count,
            "source": lib.source,
        }));
    }

    let default_name = libs.iter().find(|l| l.is_default).map(|l| l.name.clone());
    Ok(json!({
        "mode": "live",
        "total_libraries": out.len(),
        "total_books_across_libraries": total_books_across_all,
        "default_library": default_name,
        "libraries": out,
    }))
}

/// 2. Compute comprehensive statistics for a single Calibre library.
pub fn library_stats(library: Option<&str>) -> Result<Value, DeviceError> {
    match resolve_target_library(library)? {
        ResolvedLibrary::Demo => Ok(json!({
            "mode": "demo",
            "library_name": "Demo-Library",
            "path": "memory://demo",
            "has_fts_db": true,
            "total_books": 6,
            "total_authors": 6,
            "total_tags": 10,
            "total_series": 2,
            "total_publishers": 4,
            "books_with_cover": 5,
            "books_with_comments": 6,
            "formats": [
                {"format": "EPUB", "count": 5},
                {"format": "PDF", "count": 4},
                {"format": "MOBI", "count": 1}
            ]
        })),
        ResolvedLibrary::Live(lib) => {
            let conn = open_readonly_sqlite(Path::new(&lib.metadata_db))?;
            let count_query =
                |sql: &str| -> i64 { conn.query_row(sql, [], |r| r.get::<_, i64>(0)).unwrap_or(0) };
            let total_books = count_query("SELECT COUNT(*) FROM books");
            let total_authors = count_query("SELECT COUNT(*) FROM authors");
            let total_tags = count_query("SELECT COUNT(*) FROM tags");
            let total_series = count_query("SELECT COUNT(*) FROM series");
            let total_publishers = count_query("SELECT COUNT(*) FROM publishers");
            let books_with_cover = count_query("SELECT COUNT(*) FROM books WHERE has_cover = 1");
            let books_with_comments = count_query(
                "SELECT COUNT(*) FROM comments WHERE text IS NOT NULL AND trim(text) != ''",
            );

            let mut formats = Vec::new();
            if let Ok(mut stmt) = conn.prepare(
                "SELECT upper(format) AS fmt, COUNT(*) AS cnt FROM data GROUP BY upper(format) ORDER BY cnt DESC LIMIT 15",
            ) {
                if let Ok(rows) = stmt.query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                }) {
                    for pair in rows.flatten() {
                        formats.push(json!({"format": pair.0, "count": pair.1}));
                    }
                }
            }

            let size_mb = (lib.metadata_size_bytes as f64) / (1024.0 * 1024.0);
            Ok(json!({
                "mode": "live",
                "library_name": lib.name,
                "path": lib.path,
                "metadata_db": lib.metadata_db,
                "metadata_size_mb": (size_mb * 100.0).round() / 100.0,
                "has_fts_db": lib.has_fts_db,
                "total_books": total_books,
                "total_authors": total_authors,
                "total_tags": total_tags,
                "total_series": total_series,
                "total_publishers": total_publishers,
                "books_with_cover": books_with_cover,
                "books_with_comments": books_with_comments,
                "formats": formats,
            }))
        }
    }
}

/// 3. Query / filter books in a Calibre library with rich metadata joins.
pub fn query_books(params: &BookQueryParams) -> Result<Value, DeviceError> {
    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let offset = params.offset.unwrap_or(0);

    match resolve_target_library(params.library.as_deref())? {
        ResolvedLibrary::Demo => {
            let mut books = demo_books();
            if let Some(q) = params
                .query
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                let ql = q.to_lowercase();
                books.retain(|b| {
                    b.title.to_lowercase().contains(&ql)
                        || b.authors.iter().any(|a| a.to_lowercase().contains(&ql))
                        || b.tags.iter().any(|t| t.to_lowercase().contains(&ql))
                        || b.summary_snippet
                            .as_deref()
                            .unwrap_or_default()
                            .to_lowercase()
                            .contains(&ql)
                });
            }
            if let Some(a) = params
                .author
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                let al = a.to_lowercase();
                books.retain(|b| b.authors.iter().any(|x| x.to_lowercase().contains(&al)));
            }
            if let Some(t) = params
                .tag
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                let tl = t.to_lowercase();
                books.retain(|b| b.tags.iter().any(|x| x.to_lowercase().contains(&tl)));
            }
            if let Some(s) = params
                .series
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                let sl = s.to_lowercase();
                books.retain(|b| {
                    b.series
                        .as_deref()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&sl)
                });
            }
            if let Some(f) = params
                .format
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                let fu = f.to_uppercase();
                books.retain(|b| b.formats.iter().any(|x| x.eq_ignore_ascii_case(&fu)));
            }
            let total = books.len();
            let page: Vec<BookSummary> = books.into_iter().skip(offset).take(limit).collect();
            Ok(json!({
                "mode": "demo",
                "library_name": "Demo-Library",
                "total": total,
                "count": page.len(),
                "limit": limit,
                "offset": offset,
                "books": page,
            }))
        }
        ResolvedLibrary::Live(lib) => {
            let conn = open_readonly_sqlite(Path::new(&lib.metadata_db))?;
            let q = params.query.as_deref().unwrap_or("").trim().to_string();
            let author = params.author.as_deref().unwrap_or("").trim().to_string();
            let tag = params.tag.as_deref().unwrap_or("").trim().to_string();
            let series = params.series.as_deref().unwrap_or("").trim().to_string();
            let format = params.format.as_deref().unwrap_or("").trim().to_uppercase();

            let sql = r#"
                SELECT
                    b.id,
                    b.title,
                    b.path,
                    b.series_index,
                    b.has_cover,
                    CAST(b.pubdate AS TEXT) AS pubdate_str,
                    CAST(b.timestamp AS TEXT) AS timestamp_str,
                    (SELECT group_concat(a.name, '|')
                       FROM books_authors_link bal
                       JOIN authors a ON a.id = bal.author
                      WHERE bal.book = b.id) AS authors_pipe,
                    (SELECT s.name
                       FROM books_series_link bsl
                       JOIN series s ON s.id = bsl.series
                      WHERE bsl.book = b.id
                      LIMIT 1) AS series_name,
                    (SELECT group_concat(t.name, '|')
                       FROM books_tags_link btl
                       JOIN tags t ON t.id = btl.tag
                      WHERE btl.book = b.id) AS tags_pipe,
                    (SELECT group_concat(upper(d.format), '|')
                       FROM data d
                      WHERE d.book = b.id) AS formats_pipe,
                    (SELECT r.rating
                       FROM books_ratings_link brl
                       JOIN ratings r ON r.id = brl.rating
                      WHERE brl.book = b.id
                      LIMIT 1) AS rating_val,
                    (SELECT substr(c.text, 1, 400)
                       FROM comments c
                      WHERE c.book = b.id
                      LIMIT 1) AS comment_raw
                FROM books b
                WHERE (?1 = '' OR b.title LIKE '%' || ?1 || '%'
                       OR b.author_sort LIKE '%' || ?1 || '%'
                       OR EXISTS (
                           SELECT 1 FROM comments c
                            WHERE c.book = b.id AND c.text LIKE '%' || ?1 || '%'
                       ))
                  AND (?2 = '' OR EXISTS (
                       SELECT 1 FROM books_authors_link bal
                       JOIN authors a ON a.id = bal.author
                      WHERE bal.book = b.id AND a.name LIKE '%' || ?2 || '%'
                  ))
                  AND (?3 = '' OR EXISTS (
                       SELECT 1 FROM books_tags_link btl
                       JOIN tags t ON t.id = btl.tag
                      WHERE btl.book = b.id AND t.name LIKE '%' || ?3 || '%'
                  ))
                  AND (?4 = '' OR EXISTS (
                       SELECT 1 FROM books_series_link bsl
                       JOIN series s ON s.id = bsl.series
                      WHERE bsl.book = b.id AND s.name LIKE '%' || ?4 || '%'
                  ))
                  AND (?5 = '' OR EXISTS (
                       SELECT 1 FROM data d
                      WHERE d.book = b.id AND upper(d.format) = ?5
                  ))
                ORDER BY b.timestamp DESC, b.id DESC
                LIMIT ?6 OFFSET ?7
            "#;

            let mut stmt = conn
                .prepare(sql)
                .map_err(|e| DeviceError::Protocol(format!("SQLite prepare failed: {e}")))?;

            let lib_dir = Path::new(&lib.path);
            let rows = stmt
                .query_map(
                    params![q, author, tag, series, format, limit as i64, offset as i64],
                    |row| {
                        let id: i64 = row.get(0)?;
                        let title: String = row.get(1)?;
                        let rel_path: String = row.get(2)?;
                        let series_index: f64 = row.get::<_, Option<f64>>(3)?.unwrap_or(1.0);
                        let has_cover_int: i64 = row.get::<_, Option<i64>>(4)?.unwrap_or(0);
                        let pubdate: Option<String> = row.get(5)?;
                        let timestamp: Option<String> = row.get(6)?;
                        let authors_pipe: Option<String> = row.get(7)?;
                        let series_name: Option<String> = row.get(8)?;
                        let tags_pipe: Option<String> = row.get(9)?;
                        let formats_pipe: Option<String> = row.get(10)?;
                        let rating_val: Option<i64> = row.get(11)?;
                        let comment_raw: Option<String> = row.get(12)?;

                        let has_cover = has_cover_int != 0;
                        let cover_candidate = lib_dir.join(&rel_path).join("cover.jpg");
                        let cover_path = if has_cover && cover_candidate.is_file() {
                            Some(cover_candidate.to_string_lossy().into_owned())
                        } else {
                            None
                        };
                        let summary_snippet = comment_raw
                            .map(|c| strip_html_tags(&c))
                            .filter(|s| !s.is_empty())
                            .map(|s| truncate_chars(&s, 200));

                        Ok(BookSummary {
                            id,
                            title,
                            authors: split_pipe_list(authors_pipe),
                            series: series_name,
                            series_index,
                            tags: split_pipe_list(tags_pipe),
                            formats: split_pipe_list(formats_pipe),
                            rating: rating_val,
                            pubdate: pubdate.map(|s| truncate_chars(&s, 10)),
                            timestamp: timestamp.map(|s| truncate_chars(&s, 10)),
                            has_cover,
                            cover_path,
                            library_name: lib.name.clone(),
                            path: rel_path,
                            summary_snippet,
                        })
                    },
                )
                .map_err(|e| DeviceError::Protocol(format!("SQLite query failed: {e}")))?;

            let mut books = Vec::new();
            for b in rows.flatten() {
                books.push(b);
            }

            Ok(json!({
                "mode": "live",
                "library_name": lib.name,
                "library_path": lib.path,
                "count": books.len(),
                "limit": limit,
                "offset": offset,
                "books": books,
            }))
        }
    }
}

/// 4. Inspect full details, comments, identifiers, and on-disk format files for a single book ID.
pub fn book_get(library: Option<&str>, book_id: i64) -> Result<Value, DeviceError> {
    match resolve_target_library(library)? {
        ResolvedLibrary::Demo => {
            let books = demo_books();
            let b = books.into_iter().find(|x| x.id == book_id).ok_or_else(|| {
                DeviceError::InvalidInput(format!("demo book id {book_id} not found"))
            })?;
            Ok(json!({
                "mode": "demo",
                "library_name": "Demo-Library",
                "book": b,
                "comments": b.summary_snippet,
                "identifiers": [{"type": "isbn", "value": "9780441172719"}],
                "files": b.formats.iter().map(|fmt| json!({
                    "format": fmt,
                    "name": b.title,
                    "size_bytes": 1_048_576,
                    "path": format!("memory://demo/{}/{}.{}", b.path, b.title, fmt.to_lowercase()),
                    "exists": true
                })).collect::<Vec<_>>()
            }))
        }
        ResolvedLibrary::Live(lib) => {
            let conn = open_readonly_sqlite(Path::new(&lib.metadata_db))?;
            let row_res = conn.query_row(
                r#"
                SELECT
                    b.id,
                    b.title,
                    b.path,
                    b.series_index,
                    b.has_cover,
                    CAST(b.pubdate AS TEXT),
                    CAST(b.timestamp AS TEXT),
                    CAST(b.last_modified AS TEXT),
                    b.uuid,
                    (SELECT group_concat(a.name, '|') FROM books_authors_link bal JOIN authors a ON a.id = bal.author WHERE bal.book = b.id),
                    (SELECT s.name FROM books_series_link bsl JOIN series s ON s.id = bsl.series WHERE bsl.book = b.id LIMIT 1),
                    (SELECT group_concat(t.name, '|') FROM books_tags_link btl JOIN tags t ON t.id = btl.tag WHERE btl.book = b.id),
                    (SELECT p.name FROM books_publishers_link bpl JOIN publishers p ON p.id = bpl.publisher WHERE bpl.book = b.id LIMIT 1),
                    (SELECT r.rating FROM books_ratings_link brl JOIN ratings r ON r.id = brl.rating WHERE brl.book = b.id LIMIT 1),
                    (SELECT c.text FROM comments c WHERE c.book = b.id LIMIT 1)
                FROM books b
                WHERE b.id = ?1
                "#,
                params![book_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<f64>>(3)?.unwrap_or(1.0),
                        row.get::<_, Option<i64>>(4)?.unwrap_or(0) != 0,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, Option<String>>(6)?,
                        row.get::<_, Option<String>>(7)?,
                        row.get::<_, Option<String>>(8)?,
                        row.get::<_, Option<String>>(9)?,
                        row.get::<_, Option<String>>(10)?,
                        row.get::<_, Option<String>>(11)?,
                        row.get::<_, Option<String>>(12)?,
                        row.get::<_, Option<i64>>(13)?,
                        row.get::<_, Option<String>>(14)?,
                    ))
                },
            );

            let (
                id,
                title,
                rel_path,
                series_index,
                has_cover,
                pubdate,
                timestamp,
                last_modified,
                uuid,
                authors_pipe,
                series_name,
                tags_pipe,
                publisher,
                rating,
                comment_html,
            ) = row_res.map_err(|e| {
                DeviceError::InvalidInput(format!(
                    "book_id {book_id} not found in library '{}': {e}",
                    lib.name
                ))
            })?;

            let book_dir = Path::new(&lib.path).join(&rel_path);
            let cover_candidate = book_dir.join("cover.jpg");
            let cover_path = if has_cover && cover_candidate.is_file() {
                Some(cover_candidate.to_string_lossy().into_owned())
            } else {
                None
            };

            let mut identifiers = Vec::new();
            if let Ok(mut stmt) =
                conn.prepare("SELECT type, val FROM identifiers WHERE book = ?1 ORDER BY type")
            {
                if let Ok(rows) = stmt.query_map(params![book_id], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
                }) {
                    for id_pair in rows.flatten() {
                        identifiers.push(json!({"type": id_pair.0, "value": id_pair.1}));
                    }
                }
            }

            let mut files = Vec::new();
            let mut formats = Vec::new();
            if let Ok(mut stmt) = conn.prepare(
                "SELECT upper(format), uncompressed_size, name FROM data WHERE book = ?1 ORDER BY format",
            ) {
                if let Ok(rows) = stmt.query_map(params![book_id], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, Option<i64>>(1)?.unwrap_or(0),
                        r.get::<_, String>(2)?,
                    ))
                }) {
                    for (fmt, size, fname) in rows.flatten() {
                        let file_path = book_dir.join(format!("{fname}.{}", fmt.to_lowercase()));
                        let exists = file_path.is_file();
                        formats.push(fmt.clone());
                        files.push(json!({
                            "format": fmt,
                            "name": fname,
                            "size_bytes": size,
                            "path": file_path.to_string_lossy(),
                            "exists": exists,
                        }));
                    }
                }
            }

            let comment_plain = comment_html
                .as_deref()
                .map(strip_html_tags)
                .filter(|s| !s.is_empty());

            Ok(json!({
                "mode": "live",
                "library_name": lib.name,
                "library_path": lib.path,
                "book": {
                    "id": id,
                    "uuid": uuid,
                    "title": title,
                    "authors": split_pipe_list(authors_pipe),
                    "series": series_name,
                    "series_index": series_index,
                    "tags": split_pipe_list(tags_pipe),
                    "publisher": publisher,
                    "rating": rating,
                    "formats": formats,
                    "pubdate": pubdate,
                    "timestamp": timestamp,
                    "last_modified": last_modified,
                    "has_cover": has_cover,
                    "cover_path": cover_path,
                    "path": rel_path,
                    "summary_snippet": comment_plain.as_deref().map(|s| truncate_chars(s, 300)),
                },
                "comments_plain": comment_plain,
                "comments_html": comment_html,
                "identifiers": identifiers,
                "files": files,
            }))
        }
    }
}

/// Full-text search inside `full-text-search.db` (`books_text.searchable_text`), with automatic
/// fallback to metadata/comments search if `full-text-search.db` is not built for that library.
pub fn search_fulltext(
    library: Option<&str>,
    query: &str,
    limit: usize,
) -> Result<Value, DeviceError> {
    let clean_q = query.trim().trim_matches('"').trim_matches('\'');
    if clean_q.is_empty() {
        return Err(DeviceError::InvalidInput(
            "search_fulltext requires a non-empty query string".to_string(),
        ));
    }
    let capped_limit = limit.clamp(1, 100);

    match resolve_target_library(library)? {
        ResolvedLibrary::Demo => {
            let ql = clean_q.to_lowercase();
            let hits: Vec<Value> = demo_books()
                .into_iter()
                .filter(|b| {
                    b.title.to_lowercase().contains(&ql)
                        || b.summary_snippet
                            .as_deref()
                            .unwrap_or_default()
                            .to_lowercase()
                            .contains(&ql)
                })
                .take(capped_limit)
                .map(|b| {
                    json!({
                        "book_id": b.id,
                        "title": b.title,
                        "authors": b.authors,
                        "format": b.formats.first().cloned().unwrap_or_else(|| "EPUB".to_string()),
                        "snippet": b.summary_snippet,
                        "source": "demo_fts"
                    })
                })
                .collect();
            Ok(json!({
                "mode": "demo",
                "library_name": "Demo-Library",
                "query": clean_q,
                "count": hits.len(),
                "hits": hits,
            }))
        }
        ResolvedLibrary::Live(lib) => {
            let meta_conn = open_readonly_sqlite(Path::new(&lib.metadata_db))?;
            let mut hits = Vec::new();
            let mut engine = "metadata_comments_fallback";

            if let Some(fts_str) = &lib.fts_db {
                let fts_path = Path::new(fts_str);
                if let Ok(fts_conn) = open_readonly_sqlite(fts_path) {
                    engine = "full_text_search_db";
                    let sql = r#"
                        SELECT
                            book,
                            format,
                            substr(
                                searchable_text,
                                max(1, instr(lower(searchable_text), lower(?1)) - 90),
                                240
                            ) AS snippet
                        FROM books_text
                        WHERE searchable_text LIKE '%' || ?1 || '%'
                        LIMIT ?2
                    "#;
                    if let Ok(mut stmt) = fts_conn.prepare(sql) {
                        if let Ok(rows) =
                            stmt.query_map(params![clean_q, capped_limit as i64], |r| {
                                Ok((
                                    r.get::<_, i64>(0)?,
                                    r.get::<_, String>(1)?,
                                    r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                                ))
                            })
                        {
                            for (book_id, fmt, raw_snip) in rows.flatten() {
                                let (title, authors_pipe): (String, Option<String>) = meta_conn
                                    .query_row(
                                        r#"
                                        SELECT b.title,
                                               (SELECT group_concat(a.name, '|')
                                                  FROM books_authors_link bal
                                                  JOIN authors a ON a.id = bal.author
                                                 WHERE bal.book = b.id)
                                          FROM books b
                                         WHERE b.id = ?1
                                        "#,
                                        params![book_id],
                                        |mr| Ok((mr.get(0)?, mr.get(1)?)),
                                    )
                                    .unwrap_or_else(|_| (format!("Book #{book_id}"), None));

                                let clean_snip =
                                    raw_snip.split_whitespace().collect::<Vec<_>>().join(" ");
                                hits.push(json!({
                                    "book_id": book_id,
                                    "title": title,
                                    "authors": split_pipe_list(authors_pipe),
                                    "format": fmt,
                                    "snippet": truncate_chars(&clean_snip, 240),
                                    "source": "full-text-search.db:books_text",
                                }));
                            }
                        }
                    }
                }
            }

            // Fallback to comments + title search in metadata.db if FTS returned 0 or has no FTS db
            if hits.is_empty() {
                if let Ok(mut stmt) = meta_conn.prepare(
                    r#"
                    SELECT b.id,
                           b.title,
                           (SELECT group_concat(a.name, '|') FROM books_authors_link bal JOIN authors a ON a.id = bal.author WHERE bal.book = b.id),
                           (SELECT c.text FROM comments c WHERE c.book = b.id LIMIT 1)
                      FROM books b
                     WHERE b.title LIKE '%' || ?1 || '%'
                        OR EXISTS (SELECT 1 FROM comments c WHERE c.book = b.id AND c.text LIKE '%' || ?1 || '%')
                     LIMIT ?2
                    "#,
                ) {
                    if let Ok(rows) = stmt.query_map(params![clean_q, capped_limit as i64], |r| {
                        Ok((
                            r.get::<_, i64>(0)?,
                            r.get::<_, String>(1)?,
                            r.get::<_, Option<String>>(2)?,
                            r.get::<_, Option<String>>(3)?,
                        ))
                    }) {
                        for (book_id, title, authors_pipe, comment_raw) in rows.flatten() {
                            let snip = comment_raw
                                .as_deref()
                                .map(strip_html_tags)
                                .map(|s| truncate_chars(&s, 240))
                                .unwrap_or_else(|| title.clone());
                            hits.push(json!({
                                "book_id": book_id,
                                "title": title,
                                "authors": split_pipe_list(authors_pipe),
                                "format": "METADATA",
                                "snippet": snip,
                                "source": "metadata.db:comments",
                            }));
                        }
                    }
                }
            }

            Ok(json!({
                "mode": "live",
                "library_name": lib.name,
                "engine": engine,
                "query": clean_q,
                "count": hits.len(),
                "hits": hits,
            }))
        }
    }
}

/// 6. List authors with book counts.
pub fn authors_list(
    library: Option<&str>,
    query: Option<&str>,
    limit: usize,
) -> Result<Value, DeviceError> {
    let capped = limit.clamp(1, 200);
    let q = query.unwrap_or("").trim().to_string();

    match resolve_target_library(library)? {
        ResolvedLibrary::Demo => Ok(json!({
            "mode": "demo",
            "library_name": "Demo-Library",
            "authors": [
                {"id": 1, "name": "Frank Herbert", "book_count": 2},
                {"id": 2, "name": "Steve Klabnik", "book_count": 1},
                {"id": 3, "name": "William Gibson", "book_count": 1},
                {"id": 4, "name": "Martin Kleppmann", "book_count": 1},
                {"id": 5, "name": "Douglas R. Hofstadter", "book_count": 1}
            ]
        })),
        ResolvedLibrary::Live(lib) => {
            let conn = open_readonly_sqlite(Path::new(&lib.metadata_db))?;
            let mut stmt = conn
                .prepare(
                    r#"
                    SELECT a.id, a.name, COUNT(bal.book) AS book_count
                      FROM authors a
                      LEFT JOIN books_authors_link bal ON bal.author = a.id
                     WHERE (?1 = '' OR a.name LIKE '%' || ?1 || '%')
                     GROUP BY a.id, a.name
                     ORDER BY book_count DESC, a.name ASC
                     LIMIT ?2
                    "#,
                )
                .map_err(|e| DeviceError::Protocol(format!("authors prepare failed: {e}")))?;

            let mut authors = Vec::new();
            if let Ok(rows) = stmt.query_map(params![q, capped as i64], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            }) {
                for (id, name, cnt) in rows.flatten() {
                    authors.push(json!({"id": id, "name": name, "book_count": cnt}));
                }
            }

            Ok(json!({
                "mode": "live",
                "library_name": lib.name,
                "count": authors.len(),
                "authors": authors,
            }))
        }
    }
}

/// 7. List tags with book counts and unused tag count.
pub fn tags_list(
    library: Option<&str>,
    query: Option<&str>,
    limit: usize,
) -> Result<Value, DeviceError> {
    let capped = limit.clamp(1, 200);
    let q = query.unwrap_or("").trim().to_string();

    match resolve_target_library(library)? {
        ResolvedLibrary::Demo => Ok(json!({
            "mode": "demo",
            "library_name": "Demo-Library",
            "unused_tags_count": 0,
            "tags": [
                {"id": 1, "name": "Science Fiction", "book_count": 3},
                {"id": 2, "name": "Classic", "book_count": 2},
                {"id": 3, "name": "Programming", "book_count": 2},
                {"id": 4, "name": "Rust", "book_count": 1}
            ]
        })),
        ResolvedLibrary::Live(lib) => {
            let conn = open_readonly_sqlite(Path::new(&lib.metadata_db))?;
            let unused_count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM tags t WHERE NOT EXISTS (SELECT 1 FROM books_tags_link btl WHERE btl.tag = t.id)",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or(0);

            let mut stmt = conn
                .prepare(
                    r#"
                    SELECT t.id, t.name, COUNT(btl.book) AS book_count
                      FROM tags t
                      LEFT JOIN books_tags_link btl ON btl.tag = t.id
                     WHERE (?1 = '' OR t.name LIKE '%' || ?1 || '%')
                     GROUP BY t.id, t.name
                     ORDER BY book_count DESC, t.name ASC
                     LIMIT ?2
                    "#,
                )
                .map_err(|e| DeviceError::Protocol(format!("tags prepare failed: {e}")))?;

            let mut tags = Vec::new();
            if let Ok(rows) = stmt.query_map(params![q, capped as i64], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            }) {
                for (id, name, cnt) in rows.flatten() {
                    tags.push(json!({"id": id, "name": name, "book_count": cnt}));
                }
            }

            Ok(json!({
                "mode": "live",
                "library_name": lib.name,
                "unused_tags_count": unused_count,
                "count": tags.len(),
                "tags": tags,
            }))
        }
    }
}

/// 8. Series & Canon Coverage analysis (identifies series book counts and missing integer volume indices).
pub fn series_analysis(
    library: Option<&str>,
    query: Option<&str>,
    limit: usize,
) -> Result<Value, DeviceError> {
    let capped = limit.clamp(1, 200);
    let q = query.unwrap_or("").trim().to_string();

    match resolve_target_library(library)? {
        ResolvedLibrary::Demo => Ok(json!({
            "mode": "demo",
            "library_name": "Demo-Library",
            "series": [
                {
                    "id": 1,
                    "name": "Dune Chronicles",
                    "book_count": 2,
                    "min_index": 1.0,
                    "max_index": 2.0,
                    "missing_indices": []
                },
                {
                    "id": 2,
                    "name": "Sprawl Trilogy",
                    "book_count": 1,
                    "min_index": 1.0,
                    "max_index": 1.0,
                    "missing_indices": []
                }
            ]
        })),
        ResolvedLibrary::Live(lib) => {
            let conn = open_readonly_sqlite(Path::new(&lib.metadata_db))?;
            let mut stmt = conn
                .prepare(
                    r#"
                    SELECT s.id,
                           s.name,
                           COUNT(bsl.book) AS book_count,
                           MIN(b.series_index) AS min_idx,
                           MAX(b.series_index) AS max_idx,
                           group_concat(CAST(CAST(b.series_index AS INTEGER) AS TEXT), ',') AS indices_csv
                      FROM series s
                      JOIN books_series_link bsl ON bsl.series = s.id
                      JOIN books b ON b.id = bsl.book
                     WHERE (?1 = '' OR s.name LIKE '%' || ?1 || '%')
                     GROUP BY s.id, s.name
                     ORDER BY book_count DESC, s.name ASC
                     LIMIT ?2
                    "#,
                )
                .map_err(|e| DeviceError::Protocol(format!("series prepare failed: {e}")))?;

            let mut series_list = Vec::new();
            if let Ok(rows) = stmt.query_map(params![q, capped as i64], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, Option<f64>>(3)?.unwrap_or(1.0),
                    r.get::<_, Option<f64>>(4)?.unwrap_or(1.0),
                    r.get::<_, Option<String>>(5)?.unwrap_or_default(),
                ))
            }) {
                for (id, name, cnt, min_idx, max_idx, csv) in rows.flatten() {
                    let present: std::collections::BTreeSet<i64> = csv
                        .split(',')
                        .filter_map(|s| s.trim().parse::<i64>().ok())
                        .collect();
                    let max_int = (max_idx.floor() as i64).clamp(1, 200);
                    let mut missing = Vec::new();
                    for idx in 1..=max_int {
                        if !present.contains(&idx) {
                            missing.push(idx);
                        }
                    }
                    series_list.push(json!({
                        "id": id,
                        "name": name,
                        "book_count": cnt,
                        "min_index": min_idx,
                        "max_index": max_idx,
                        "missing_indices": missing,
                    }));
                }
            }

            Ok(json!({
                "mode": "live",
                "library_name": lib.name,
                "count": series_list.len(),
                "series": series_list,
            }))
        }
    }
}

/// 9. Find duplicate books sharing normalized titles inside a library.
pub fn find_duplicates(library: Option<&str>, limit: usize) -> Result<Value, DeviceError> {
    let capped = limit.clamp(1, 100);
    match resolve_target_library(library)? {
        ResolvedLibrary::Demo => Ok(json!({
            "mode": "demo",
            "library_name": "Demo-Library",
            "duplicate_groups": []
        })),
        ResolvedLibrary::Live(lib) => {
            let conn = open_readonly_sqlite(Path::new(&lib.metadata_db))?;
            let mut stmt = conn
                .prepare(
                    r#"
                    SELECT lower(trim(b.title)) AS norm_title,
                           COUNT(*) AS dup_count,
                           group_concat(b.id, '|') AS book_ids,
                           MAX(b.title) AS display_title
                      FROM books b
                     GROUP BY lower(trim(b.title))
                    HAVING COUNT(*) > 1
                     ORDER BY dup_count DESC, display_title ASC
                     LIMIT ?1
                    "#,
                )
                .map_err(|e| DeviceError::Protocol(format!("duplicates prepare failed: {e}")))?;

            let mut groups = Vec::new();
            if let Ok(rows) = stmt.query_map(params![capped as i64], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, String>(3)?,
                ))
            }) {
                for (norm, cnt, ids_pipe, display_title) in rows.flatten() {
                    groups.push(json!({
                        "title": display_title,
                        "normalized_title": norm,
                        "count": cnt,
                        "book_ids": split_pipe_list(ids_pipe),
                    }));
                }
            }

            Ok(json!({
                "mode": "live",
                "library_name": lib.name,
                "duplicate_group_count": groups.len(),
                "duplicate_groups": groups,
            }))
        }
    }
}

/// 10. Library Health & Metadata Hygiene Audit (`analyze_library_health` / `reading_statistics`).
pub fn library_health(library: Option<&str>) -> Result<Value, DeviceError> {
    let stats = library_stats(library)?;
    let dups = find_duplicates(library, 25)?;

    let total_books = stats
        .get("total_books")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let with_cover = stats
        .get("books_with_cover")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let with_comments = stats
        .get("books_with_comments")
        .and_then(Value::as_i64)
        .unwrap_or(0);

    let missing_covers = total_books.saturating_sub(with_cover);
    let missing_comments = total_books.saturating_sub(with_comments);
    let dup_groups = dups
        .get("duplicate_group_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);

    let cover_pct = if total_books > 0 {
        ((with_cover as f64) / (total_books as f64) * 1000.0).round() / 10.0
    } else {
        100.0
    };
    let comment_pct = if total_books > 0 {
        ((with_comments as f64) / (total_books as f64) * 1000.0).round() / 10.0
    } else {
        100.0
    };

    Ok(json!({
        "library_name": stats.get("library_name"),
        "mode": stats.get("mode"),
        "total_books": total_books,
        "missing_covers": missing_covers,
        "missing_comments": missing_comments,
        "cover_coverage_pct": cover_pct,
        "comment_coverage_pct": comment_pct,
        "duplicate_title_groups": dup_groups,
        "stats": stats,
        "top_duplicates": dups.get("duplicate_groups"),
    }))
}

/// Group books by primary tag (`bookcase_catalog` visual shelf view).
pub fn bookcase_catalog(library: Option<&str>, limit: usize) -> Result<Value, DeviceError> {
    let params = BookQueryParams {
        library: library.map(ToString::to_string),
        limit: Some(limit.clamp(1, 200)),
        ..Default::default()
    };
    let res = query_books(&params)?;
    let mut shelves: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    if let Some(arr) = res.get("books").and_then(Value::as_array) {
        for b in arr {
            let shelf_name = b
                .get("tags")
                .and_then(Value::as_array)
                .and_then(|t| t.first())
                .and_then(Value::as_str)
                .unwrap_or("Unshelved")
                .to_string();
            shelves.entry(shelf_name).or_default().push(b.clone());
        }
    }
    Ok(json!({
        "library_name": res.get("library_name"),
        "mode": res.get("mode"),
        "shelf_count": shelves.len(),
        "shelves": shelves,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_query_and_utf8_truncation_are_infallible() {
        let truncated = truncate_chars("Gödel, Escher, Bach: ÄÖÜß", 10);
        assert!(truncated.starts_with("Gödel"));

        let res = query_books(&BookQueryParams {
            library: Some("demo".to_string()),
            query: Some("Gödel".to_string()),
            ..Default::default()
        });
        assert!(res.is_ok());
        if let Ok(v) = res {
            assert_eq!(v["count"], 1);
        }
    }

    #[test]
    fn demo_book_get_and_health_succeed() {
        let book = book_get(Some("demo"), 1);
        assert!(book.is_ok());
        let health = library_health(Some("demo"));
        assert!(health.is_ok());
    }
}
