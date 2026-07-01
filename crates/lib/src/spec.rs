use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use walkdir::WalkDir;

mod parser;
mod types;

pub use parser::{normalize_to_pointer, parse_spec};
pub use types::{ContextWindow, SourceSpan, SpecIndex};

/// Resolve a spec path (from config or user input) to a path relative to the project root.
pub fn normalize_spec_path(root: &Path, spec: &str) -> Result<PathBuf> {
    if spec.trim().is_empty() {
        bail!("Spec path cannot be blank");
    }
    let spec_path = PathBuf::from(spec);
    let absolute = if spec_path.is_absolute() {
        spec_path
    } else {
        root.join(&spec_path)
    };
    if !absolute.is_file() {
        bail!("Spec file not found: {}", absolute.display());
    }
    let relative = absolute
        .strip_prefix(root)
        .context("Spec path must be inside the project root")?;
    Ok(relative.to_path_buf())
}

/// Walk the directory tree to find OpenAPI spec files.
/// Returns a sorted list of relative paths.
pub fn discover_spec(root: &Path, max_depth: usize) -> Result<Vec<String>> {
    // Check well-known names first; trust by name, no content check.
    for name in ["openapi.yaml", "openapi.yml", "openapi.json"] {
        if root.join(name).is_file() {
            return Ok(vec![name.to_string()]);
        }
    }

    let mut matches = Vec::new();
    let walker = WalkDir::new(root)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !should_skip_entry(e));

    for entry in walker.filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if is_spec_file(path)
            && is_openapi_spec(path)
            && let Ok(rel) = path.strip_prefix(root)
        {
            matches.push(rel.to_string_lossy().to_string());
        }
    }

    matches.sort();
    Ok(matches)
}

/// Check whether content (as a string) looks like an OpenAPI spec.
/// For JSON content, looks for `"openapi":` as a key. For YAML, looks for `openapi:`.
/// Only inspects the first 512 characters.
pub fn looks_like_openapi(content: &str, json: bool) -> bool {
    let head: String = content.chars().take(512).collect();
    if json {
        let stripped: String = head.chars().filter(|c| !c.is_whitespace()).collect();
        stripped.contains("\"openapi\":")
    } else {
        head.contains("openapi:")
    }
}

fn is_spec_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_lowercase()),
        Some(ext) if ext == "yaml" || ext == "yml" || ext == "json"
    )
}

fn is_json(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_lowercase()),
        Some(ext) if ext == "json"
    )
}

/// Two-stage check: fast 512-byte heuristic, then a full yaml_serde parse confirming
/// a top-level mapping with an `openapi` key. Returns false on any I/O or parse failure.
/// JSON is valid YAML, so yaml_serde handles both formats.
fn is_openapi_spec(path: &Path) -> bool {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut head = [0u8; 512];
    let n = match file.read(&mut head) {
        Ok(n) => n,
        Err(_) => return false,
    };
    let prefix = String::from_utf8_lossy(&head[..n]);
    if !looks_like_openapi(&prefix, is_json(path)) {
        return false;
    }

    // Full parse to confirm it's a valid mapping with an `openapi` key.
    let mut content = prefix.into_owned();
    if file.read_to_string(&mut content).is_err() {
        return false;
    }
    let doc: yaml_serde::Value = match yaml_serde::from_str(&content) {
        Ok(v) => v,
        Err(_) => return false,
    };
    match doc {
        yaml_serde::Value::Mapping(mapping) => mapping
            .keys()
            .filter_map(|k| k.as_str())
            .any(|k| k == "openapi"),
        _ => false,
    }
}

fn should_skip_entry(entry: &walkdir::DirEntry) -> bool {
    if entry.depth() == 0 || !entry.file_type().is_dir() {
        return false;
    }
    matches!(
        entry.file_name().to_str().unwrap_or_default(),
        ".git" | ".oav" | "target" | "node_modules" | ".idea" | ".vscode"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // looks_like_openapi

    #[test]
    fn looks_like_openapi_yaml_positive() {
        assert!(looks_like_openapi(
            "openapi: 3.0.3\ninfo:\n  title: Test",
            false
        ));
    }

    #[test]
    fn looks_like_openapi_yaml_negative() {
        assert!(!looks_like_openapi("name: not a spec\nversion: 1.0", false));
    }

    #[test]
    fn looks_like_openapi_json_positive() {
        assert!(looks_like_openapi(
            r#"{"openapi": "3.0.3", "info": {}}"#,
            true
        ));
    }

    #[test]
    fn looks_like_openapi_json_negative() {
        assert!(!looks_like_openapi(r#"{"name": "not a spec"}"#, true));
    }

    #[test]
    fn looks_like_openapi_json_value_not_key() {
        // "openapi" as a value, not a key — should not match
        assert!(!looks_like_openapi(
            r#"{"type": "openapi", "name": "tool"}"#,
            true
        ));
    }

    // Discovery

    #[test]
    fn discover_finds_well_known_yaml() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("openapi.yaml"),
            "openapi: 3.0.0\ninfo:\n  title: Test\n  version: '1.0'\npaths: {}\n",
        )
        .unwrap();

        let specs = discover_spec(dir.path(), 4).unwrap();
        assert_eq!(specs, vec!["openapi.yaml"]);
    }

    #[test]
    fn discover_finds_well_known_json() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("openapi.json"),
            r#"{"openapi":"3.0.0","info":{"title":"Test","version":"1.0"},"paths":{}}"#,
        )
        .unwrap();

        let specs = discover_spec(dir.path(), 4).unwrap();
        assert_eq!(specs, vec!["openapi.json"]);
    }

    #[test]
    fn discover_finds_nested_spec() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("api");
        fs::create_dir_all(&sub).unwrap();
        fs::write(
            sub.join("spec.yml"),
            "openapi: 3.1.0\ninfo:\n  title: Nested\n  version: '1.0'\npaths: {}\n",
        )
        .unwrap();

        let specs = discover_spec(dir.path(), 4).unwrap();
        assert_eq!(specs, vec!["api/spec.yml"]);
    }

    #[test]
    fn discover_finds_nested_json_spec() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("api");
        fs::create_dir_all(&sub).unwrap();
        fs::write(
            sub.join("spec.json"),
            r#"{"openapi":"3.0.0","info":{"title":"Test","version":"1.0"},"paths":{}}"#,
        )
        .unwrap();

        let specs = discover_spec(dir.path(), 4).unwrap();
        assert_eq!(specs, vec!["api/spec.json"]);
    }

    #[test]
    fn discover_ignores_non_openapi_yaml() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("config.yaml"),
            "database:\n  host: localhost\n",
        )
        .unwrap();

        let specs = discover_spec(dir.path(), 4).unwrap();
        assert!(specs.is_empty());
    }

    #[test]
    fn discover_skips_ignored_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let hidden = dir.path().join("node_modules").join("some-package");
        fs::create_dir_all(&hidden).unwrap();
        fs::write(
            hidden.join("openapi.yaml"),
            "openapi: 3.0.0\ninfo:\n  title: Hidden\n  version: '1.0'\npaths: {}\n",
        )
        .unwrap();

        let specs = discover_spec(dir.path(), 4).unwrap();
        assert!(specs.is_empty());
    }

    // normalize_spec_path

    #[test]
    fn normalize_rejects_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let result = normalize_spec_path(dir.path(), "nonexistent.yaml");
        assert!(result.is_err());
    }

    #[test]
    fn normalize_rejects_blank_path() {
        let dir = tempfile::tempdir().unwrap();
        let result = normalize_spec_path(dir.path(), "   ");
        assert!(result.is_err());
    }

    #[test]
    fn normalize_resolves_relative_path() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("api.yaml"), "openapi: 3.0.0\n").unwrap();

        let path = normalize_spec_path(dir.path(), "api.yaml").unwrap();
        assert_eq!(path, PathBuf::from("api.yaml"));
    }
}
