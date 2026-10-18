//! Custom Calibre library registry: user-pinned library paths.
//!
//! Stored as plain JSON next to `ui.json`. Corrupt file -> empty registry, never a crash.
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unimplemented)]
#![deny(clippy::todo)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegLibrary {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Registry {
    pub libraries: Vec<RegLibrary>,
}

pub fn path() -> Option<std::path::PathBuf> {
    std::env::var("LOCALAPPDATA").ok().map(|base| {
        std::path::PathBuf::from(base)
            .join("rust-calibre-mcp")
            .join("libraries.json")
    })
}

pub fn load() -> Registry {
    let Some(path) = path() else {
        return Registry::default();
    };
    let text = std::fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str(&text).unwrap_or_default()
}

pub fn save(registry: &Registry) {
    let Some(path) = path() else { return };
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            let _ = std::fs::create_dir_all(parent);
        }
    }
    if let Ok(text) = serde_json::to_string_pretty(registry) {
        let _ = std::fs::write(path, text);
    }
}

pub fn pin_active(name: &str, lib_path: &str) -> Registry {
    let mut reg = load();
    if !reg.libraries.iter().any(|l| l.name == name) {
        reg.libraries.push(RegLibrary {
            name: name.to_string(),
            path: lib_path.to_string(),
        });
        save(&reg);
    }
    reg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corrupt_file_is_empty_registry() {
        let registry: Registry = serde_json::from_str("not json{{").unwrap_or_default();
        assert!(registry.libraries.is_empty());
    }
}
