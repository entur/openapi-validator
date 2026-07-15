use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

/// The `.oav/` working directory name.
pub const OAV_DIR: &str = ".oav";

/// Header line written above oav-managed entries in `.gitignore`.
pub const GITIGNORE_HEADER: &str = "# openapi-validator";

/// Standard workspace entry: ignore the whole `.oav/` directory tree.
pub const WORKSPACE_GITIGNORE_ENTRIES: &[&str] = &[".oav/"];

/// Relative path of the compose file the pipeline's compile phase runs against.
pub const DOCKER_COMPOSE_FILE: &str = ".oav/docker-compose.yaml";

/// Directories the pipeline expects to exist under the workspace root.
pub const RUNTIME_DIRS: &[&str] = &[
    ".oav/configs",
    ".oav/generated",
    ".oav/reports/lint",
    ".oav/reports/generate/server",
    ".oav/reports/generate/client",
    ".oav/reports/compile/server",
    ".oav/reports/compile/client",
];

const DOCKER_COMPOSE_YAML: &str = include_str!("../assets/docker-compose.yaml");

/// Prepare the `.oav/` workspace: create the runtime directory tree and write
/// the docker-compose file if missing. Idempotent; an existing compose file is
/// never overwritten.
pub fn prepare_workspace(root: &Path) -> Result<()> {
    ensure_dirs(root, RUNTIME_DIRS)?;

    let compose_path = root.join(DOCKER_COMPOSE_FILE);
    if !compose_path.exists() {
        fs::write(&compose_path, DOCKER_COMPOSE_YAML)
            .with_context(|| format!("failed to write {}", compose_path.display()))?;
    }

    Ok(())
}

/// Create each directory in `dirs` (relative to `root`) using `create_dir_all`.
pub fn ensure_dirs(root: &Path, dirs: &[&str]) -> Result<()> {
    for dir in dirs {
        let path = root.join(dir);
        fs::create_dir_all(&path)
            .with_context(|| format!("failed to create {}", path.display()))?;
    }
    Ok(())
}

/// Append any of `entries` not already present to `root/.gitignore`.
///
/// When `header` is `Some` and at least one entry is added, the header is written
/// once before the new block (blank-line separated from existing content). If the
/// header line is already present in the file, it is not duplicated. No file write
/// occurs when nothing new is added.
pub fn add_gitignore_entries(root: &Path, entries: &[&str], header: Option<&str>) -> Result<()> {
    let path = root.join(".gitignore");

    let content = if path.exists() {
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?
    } else {
        String::new()
    };

    let missing: Vec<&str> = entries
        .iter()
        .copied()
        .filter(|e| !content.lines().any(|line| line.trim() == *e))
        .collect();

    if missing.is_empty() {
        return Ok(());
    }

    let mut appendix = String::new();

    if let Some(hdr) = header {
        let header_present = content.lines().any(|line| line.trim() == hdr);
        if !header_present {
            if content.is_empty() {
                appendix.push_str(hdr);
                appendix.push('\n');
            } else {
                if !content.ends_with('\n') {
                    appendix.push('\n');
                }
                appendix.push('\n');
                appendix.push_str(hdr);
                appendix.push('\n');
            }
        } else {
            // Header already present; just ensure we start on a new line.
            if !content.ends_with('\n') {
                appendix.push('\n');
            }
        }
    } else if !content.ends_with('\n') && !content.is_empty() {
        appendix.push('\n');
    }

    for entry in &missing {
        appendix.push_str(entry);
        appendix.push('\n');
    }

    fs::write(&path, format!("{content}{appendix}"))
        .with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}

/// Remove any lines from `root/.gitignore` whose trimmed value matches an entry in `entries`.
pub fn remove_gitignore_entries(root: &Path, entries: &[&str]) -> Result<()> {
    let path = root.join(".gitignore");
    if !path.exists() {
        return Ok(());
    }
    let content =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let kept: Vec<&str> = content
        .lines()
        .filter(|line| !entries.iter().any(|entry| line.trim() == *entry))
        .collect();
    let mut new_content = kept.join("\n");
    if !new_content.is_empty() {
        new_content.push('\n');
    }
    fs::write(&path, new_content).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

/// Ensure the standard `.oav/` workspace entry is present in `.gitignore`,
/// under the `# openapi-validator` header.
pub fn ensure_workspace_gitignore(root: &Path) -> Result<()> {
    add_gitignore_entries(root, WORKSPACE_GITIGNORE_ENTRIES, Some(GITIGNORE_HEADER))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // --- ensure_dirs ---

    #[test]
    fn ensure_dirs_creates_nested_tree() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = &[".oav/generated", ".oav/reports/lint", ".oav/configs"];
        ensure_dirs(tmp.path(), dirs).unwrap();
        for dir in dirs {
            assert!(tmp.path().join(dir).is_dir(), "{dir} not created");
        }
    }

    #[test]
    fn ensure_dirs_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = &[".oav/generated"];
        ensure_dirs(tmp.path(), dirs).unwrap();
        ensure_dirs(tmp.path(), dirs).unwrap(); // second call must not error
        assert!(tmp.path().join(".oav/generated").is_dir());
    }

    // --- add_gitignore_entries ---

    #[test]
    fn add_entries_creates_file_with_header() {
        let tmp = tempfile::tempdir().unwrap();
        add_gitignore_entries(tmp.path(), &[".oav/"], Some(GITIGNORE_HEADER)).unwrap();

        let content = fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
        assert!(content.contains(GITIGNORE_HEADER), "header missing");
        assert!(content.contains(".oav/"), "entry missing");
        // Header comes before the entry.
        let header_pos = content.find(GITIGNORE_HEADER).unwrap();
        let entry_pos = content.find(".oav/").unwrap();
        assert!(header_pos < entry_pos);
    }

    #[test]
    fn add_entries_appends_with_blank_separator() {
        let tmp = tempfile::tempdir().unwrap();
        let gi = tmp.path().join(".gitignore");
        fs::write(&gi, "node_modules/\n").unwrap();

        add_gitignore_entries(tmp.path(), &[".oav/"], Some(GITIGNORE_HEADER)).unwrap();

        let content = fs::read_to_string(&gi).unwrap();
        assert!(content.starts_with("node_modules/\n"));
        // A blank line separates existing content from the new block.
        assert!(content.contains("\n\n"), "expected blank separator");
        assert!(content.contains(GITIGNORE_HEADER));
        assert!(content.contains(".oav/"));
    }

    #[test]
    fn add_entries_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        add_gitignore_entries(tmp.path(), &[".oav/"], Some(GITIGNORE_HEADER)).unwrap();
        add_gitignore_entries(tmp.path(), &[".oav/"], Some(GITIGNORE_HEADER)).unwrap();

        let content = fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
        assert_eq!(
            content.matches(GITIGNORE_HEADER).count(),
            1,
            "header duplicated"
        );
        assert_eq!(content.matches(".oav/").count(), 1, "entry duplicated");
    }

    #[test]
    fn add_entries_no_header() {
        let tmp = tempfile::tempdir().unwrap();
        let gi = tmp.path().join(".gitignore");
        fs::write(&gi, "node_modules/\n").unwrap();

        add_gitignore_entries(tmp.path(), &[".oavc"], None).unwrap();

        let content = fs::read_to_string(&gi).unwrap();
        assert!(content.contains("node_modules/"));
        assert!(content.contains(".oavc"));
        assert!(!content.contains(GITIGNORE_HEADER));
    }

    #[test]
    fn add_entries_no_write_when_all_present() {
        let tmp = tempfile::tempdir().unwrap();
        let gi = tmp.path().join(".gitignore");
        fs::write(&gi, ".oav/\n").unwrap();
        let before = fs::metadata(&gi).unwrap().modified().unwrap();

        // Sleep briefly so mtime would differ if written.
        std::thread::sleep(std::time::Duration::from_millis(10));
        add_gitignore_entries(tmp.path(), &[".oav/"], None).unwrap();

        let after = fs::metadata(&gi).unwrap().modified().unwrap();
        assert_eq!(
            before, after,
            "file was written when no changes were needed"
        );
    }

    #[test]
    fn add_entries_header_not_duplicated_when_already_present() {
        let tmp = tempfile::tempdir().unwrap();
        let gi = tmp.path().join(".gitignore");
        // Pre-existing file already has the header but not the entry.
        fs::write(&gi, format!("{GITIGNORE_HEADER}\n")).unwrap();

        add_gitignore_entries(tmp.path(), &[".oav/"], Some(GITIGNORE_HEADER)).unwrap();

        let content = fs::read_to_string(&gi).unwrap();
        assert_eq!(
            content.matches(GITIGNORE_HEADER).count(),
            1,
            "header duplicated"
        );
        assert!(content.contains(".oav/"));
    }

    // --- remove_gitignore_entries ---

    #[test]
    fn remove_entries_drops_matching_lines() {
        let tmp = tempfile::tempdir().unwrap();
        let gi = tmp.path().join(".gitignore");
        fs::write(&gi, "node_modules/\n.oav/\n.oavc\n").unwrap();

        remove_gitignore_entries(tmp.path(), &[".oav/", ".oavc"]).unwrap();

        let content = fs::read_to_string(&gi).unwrap();
        assert!(!content.contains(".oav/"));
        assert!(!content.contains(".oavc"));
        assert!(content.contains("node_modules/"));
        assert!(content.ends_with('\n'));
    }

    #[test]
    fn remove_entries_noop_when_file_missing() {
        let tmp = tempfile::tempdir().unwrap();
        // Should not error when .gitignore doesn't exist.
        remove_gitignore_entries(tmp.path(), &[".oav/"]).unwrap();
    }

    #[test]
    fn remove_entries_preserves_trailing_newline() {
        let tmp = tempfile::tempdir().unwrap();
        let gi = tmp.path().join(".gitignore");
        fs::write(&gi, "node_modules/\n.oav/\n").unwrap();

        remove_gitignore_entries(tmp.path(), &[".oav/"]).unwrap();

        let content = fs::read_to_string(&gi).unwrap();
        assert_eq!(content, "node_modules/\n");
    }

    // --- ensure_workspace_gitignore ---

    #[test]
    fn ensure_workspace_gitignore_writes_oav_entry() {
        let tmp = tempfile::tempdir().unwrap();
        ensure_workspace_gitignore(tmp.path()).unwrap();

        let content = fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
        assert!(content.contains(GITIGNORE_HEADER));
        assert!(content.contains(".oav/"));
    }

    #[test]
    fn ensure_workspace_gitignore_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        ensure_workspace_gitignore(tmp.path()).unwrap();
        ensure_workspace_gitignore(tmp.path()).unwrap();

        let content = fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
        assert_eq!(content.matches(GITIGNORE_HEADER).count(), 1);
        assert_eq!(content.matches(".oav/").count(), 1);
    }

    // --- prepare_workspace ---

    #[test]
    fn prepare_workspace_creates_runtime_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        prepare_workspace(tmp.path()).unwrap();
        for dir in RUNTIME_DIRS {
            assert!(tmp.path().join(dir).is_dir(), "{dir} not created");
        }
    }

    #[test]
    fn prepare_workspace_writes_compose_file() {
        let tmp = tempfile::tempdir().unwrap();
        prepare_workspace(tmp.path()).unwrap();
        let content = fs::read_to_string(tmp.path().join(DOCKER_COMPOSE_FILE)).unwrap();
        assert!(content.contains("build-spring"));
        assert!(content.contains("build-client-typescript-axios"));
    }

    #[test]
    fn prepare_workspace_preserves_existing_compose() {
        let tmp = tempfile::tempdir().unwrap();
        let compose = tmp.path().join(DOCKER_COMPOSE_FILE);
        fs::create_dir_all(compose.parent().unwrap()).unwrap();
        fs::write(&compose, "custom content").unwrap();

        prepare_workspace(tmp.path()).unwrap();

        let content = fs::read_to_string(&compose).unwrap();
        assert_eq!(content, "custom content");
    }

    #[test]
    fn prepare_workspace_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        prepare_workspace(tmp.path()).unwrap();
        prepare_workspace(tmp.path()).unwrap();
        assert!(tmp.path().join(DOCKER_COMPOSE_FILE).is_file());
    }
}
