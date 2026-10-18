//! Zero-config importer for legacy `calibre-mcp` (`config.json` & `.env`),
//! standard OS Calibre configuration (`%APPDATA%\calibre\gui.json` & `global.py.json`),
//! and automatic discovery of Calibre library directories (`metadata.db` + `full-text-search.db`).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Discovered Calibre library on disk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveredLibrary {
    pub name: String,
    pub path: String,
    pub metadata_db: String,
    pub fts_db: Option<String>,
    pub metadata_size_bytes: u64,
    pub has_fts_db: bool,
    pub is_default: bool,
    pub source: String,
}

/// Candidate sibling paths for legacy `calibre-mcp`.
fn legacy_repo_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(explicit) = std::env::var("CALIBRE_MCP_LEGACY_DIR") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            out.push(PathBuf::from(trimmed));
        }
    }
    out.push(PathBuf::from("../calibre-mcp"));
    out
}

/// Parse a `.env` file into key/value pairs (never logging secret values).
pub fn parse_dotenv(text: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let stripped = line.strip_prefix("export ").unwrap_or(line).trim();
        if let Some((k, v)) = stripped.split_once('=') {
            let key = k.trim().to_string();
            let mut val = v.trim().to_string();
            if (val.starts_with('"') && val.ends_with('"'))
                || (val.starts_with('\'') && val.ends_with('\''))
            {
                val = val
                    .chars()
                    .skip(1)
                    .take(val.chars().count().saturating_sub(2))
                    .collect();
            }
            if !key.is_empty() {
                map.insert(key, val);
            }
        }
    }
    map
}

/// Resolve a configuration key from explicit argument -> process env -> sibling `.env`.
pub fn resolve(key: &str, explicit: Option<&str>) -> Option<String> {
    if let Some(exp) = explicit {
        let t = exp.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    if let Ok(val) = std::env::var(key) {
        let t = val.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    for root in legacy_repo_candidates() {
        let env_path = root.join(".env");
        if let Ok(content) = std::fs::read_to_string(&env_path) {
            let map = parse_dotenv(&content);
            if let Some(v) = map.get(key) {
                if !v.trim().is_empty() {
                    return Some(v.trim().to_string());
                }
            }
        }
    }
    None
}

/// Look up a secret in the sibling `calibre-mcp/.env` file if present.
pub fn fleet_secret(name: &str) -> Option<String> {
    resolve(name, None)
}

fn inspect_library_dir(
    name: &str,
    dir: &Path,
    is_default: bool,
    source: &str,
) -> Option<DiscoveredLibrary> {
    let meta = dir.join("metadata.db");
    let meta_stat = std::fs::metadata(&meta).ok()?;
    if !meta_stat.is_file() || meta_stat.len() == 0 {
        return None;
    }
    let fts = dir.join("full-text-search.db");
    let has_fts = fts.is_file();
    Some(DiscoveredLibrary {
        name: name.to_string(),
        path: dir.to_string_lossy().into_owned(),
        metadata_db: meta.to_string_lossy().into_owned(),
        fts_db: if has_fts {
            Some(fts.to_string_lossy().into_owned())
        } else {
            None
        },
        metadata_size_bytes: meta_stat.len(),
        has_fts_db: has_fts,
        is_default,
        source: source.to_string(),
    })
}

/// Discover all Calibre libraries across explicit env, `%APPDATA%\calibre`,
/// sibling `calibre-mcp/config.json`, and parent sibling directories.
pub fn discover_libraries() -> Vec<DiscoveredLibrary> {
    let mut by_name: BTreeMap<String, DiscoveredLibrary> = BTreeMap::new();
    let mut default_path: Option<String> = None;
    let mut parent_roots: BTreeSet<PathBuf> = BTreeSet::new();

    // 1. Environment variable override (`CALIBRE_LIBRARY_PATH` or `CALIBRE_BASE_PATH`)
    for key in ["CALIBRE_LIBRARY_PATH", "CALIBRE_BASE_PATH"] {
        if let Some(raw_path) = resolve(key, None) {
            let p = PathBuf::from(&raw_path);
            let name = p
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "EnvLibrary".to_string());
            if let Some(lib) = inspect_library_dir(&name, &p, true, key) {
                default_path = Some(lib.path.clone());
                if let Some(parent) = p.parent() {
                    parent_roots.insert(parent.to_path_buf());
                }
                by_name.insert(lib.name.clone(), lib);
            } else if p.is_dir() {
                parent_roots.insert(p);
            }
        }
    }

    // 2. Standard OS Calibre config (`%APPDATA%\calibre\gui.json` and `global.py.json`)
    if let Ok(appdata) = std::env::var("APPDATA") {
        let cal_dir = PathBuf::from(appdata).join("calibre");
        for cfg_name in ["gui.json", "global.py.json"] {
            let p = cal_dir.join(cfg_name);
            if let Ok(raw) = std::fs::read_to_string(&p) {
                if let Ok(val) = serde_json::from_str::<Value>(&raw) {
                    if let Some(lp) = val.get("library_path").and_then(Value::as_str) {
                        let lp_buf = PathBuf::from(lp);
                        let name = lp_buf
                            .file_name()
                            .map(|s| s.to_string_lossy().into_owned())
                            .unwrap_or_else(|| "DefaultLibrary".to_string());
                        if let Some(lib) = inspect_library_dir(
                            &name,
                            &lp_buf,
                            default_path.is_none(),
                            "appdata:calibre",
                        ) {
                            if default_path.is_none() {
                                default_path = Some(lib.path.clone());
                            }
                            if let Some(parent) = lp_buf.parent() {
                                parent_roots.insert(parent.to_path_buf());
                            }
                            by_name.entry(lib.name.clone()).or_insert(lib);
                        }
                    }
                    if let Some(stats) = val.get("library_usage_stats").and_then(Value::as_object) {
                        for k in stats.keys() {
                            let lp_buf = PathBuf::from(k);
                            let name = lp_buf
                                .file_name()
                                .map(|s| s.to_string_lossy().into_owned())
                                .unwrap_or_else(|| "Library".to_string());
                            let is_def = default_path.as_deref() == Some(k.as_str());
                            if let Some(lib) = inspect_library_dir(
                                &name,
                                &lp_buf,
                                is_def,
                                "appdata:calibre:library_usage_stats",
                            ) {
                                if let Some(parent) = lp_buf.parent() {
                                    parent_roots.insert(parent.to_path_buf());
                                }
                                by_name.entry(lib.name.clone()).or_insert(lib);
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Sibling `calibre-mcp/config.json`
    for root in legacy_repo_candidates() {
        let cfg_path = root.join("config.json");
        if let Ok(raw) = std::fs::read_to_string(&cfg_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&raw) {
                if let Some(loc) = val.get("local_library_path").and_then(Value::as_str) {
                    let loc_path = PathBuf::from(loc);
                    let loc_name = loc_path
                        .file_name()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "DefaultLibrary".to_string());
                    if let Some(lib) = inspect_library_dir(
                        &loc_name,
                        &loc_path,
                        default_path.is_none(),
                        "calibre-mcp/config.json:local_library_path",
                    ) {
                        if default_path.is_none() {
                            default_path = Some(lib.path.clone());
                        }
                        if let Some(parent) = loc_path.parent() {
                            parent_roots.insert(parent.to_path_buf());
                        }
                        by_name.entry(lib.name.clone()).or_insert(lib);
                    }
                }
                if let Some(disc) = val.get("discovered_libraries").and_then(Value::as_object) {
                    for (k, entry) in disc {
                        if let Some(p_str) = entry.get("path").and_then(Value::as_str) {
                            let p = PathBuf::from(p_str);
                            let is_def = default_path.as_deref() == Some(p_str);
                            if let Some(lib) = inspect_library_dir(
                                k,
                                &p,
                                is_def,
                                "calibre-mcp/config.json:discovered_libraries",
                            ) {
                                if let Some(parent) = p.parent() {
                                    parent_roots.insert(parent.to_path_buf());
                                }
                                by_name.entry(lib.name.clone()).or_insert(lib);
                            }
                        }
                    }
                }
            }
            break;
        }
    }

    // 4. Scan sibling directories of any discovered library root
    for parent in parent_roots {
        if let Ok(entries) = std::fs::read_dir(&parent) {
            for entry in entries.flatten() {
                let sub = entry.path();
                if sub.is_dir() {
                    let sub_name = entry.file_name().to_string_lossy().into_owned();
                    let is_def = default_path.is_none();
                    if let Some(lib) =
                        inspect_library_dir(&sub_name, &sub, is_def, "sibling_library_scan")
                    {
                        if is_def {
                            default_path = Some(lib.path.clone());
                        }
                        by_name.entry(lib.name.clone()).or_insert(lib);
                    }
                }
            }
        }
    }

    // 5. Ensure at least one library is marked default if any exist
    let mut list: Vec<DiscoveredLibrary> = by_name.into_values().collect();
    if !list.iter().any(|l| l.is_default) {
        if let Some(first) = list.first_mut() {
            first.is_default = true;
        }
    }
    list
}

/// Summary report of imported legacy configuration and discovered libraries.
pub fn import_summary() -> Value {
    let libs = discover_libraries();
    let default_lib = libs.iter().find(|l| l.is_default).map(|l| l.name.clone());
    json!({
        "discovered_count": libs.len(),
        "default_library": default_lib,
        "libraries": libs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dotenv_parser_handles_quotes_and_comments() {
        let sample =
            "# comment\nCALIBRE_LIBRARY_PATH=\"/books/MainLibrary\"\nCALIBRE_TIMEOUT=30\n";
        let map = parse_dotenv(sample);
        assert_eq!(map.get("CALIBRE_TIMEOUT").map(String::as_str), Some("30"));
        assert!(map.contains_key("CALIBRE_LIBRARY_PATH"));
    }
}
