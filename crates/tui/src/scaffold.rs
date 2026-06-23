use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

const OAV_DIRS: &[&str] = &[
    ".oav/configs",
    ".oav/generated",
    ".oav/reports/lint",
    ".oav/reports/generate",
    ".oav/reports/compile",
];

const DOCKER_COMPOSE_YAML: &str = include_str!("../assets/docker-compose.yaml");

/// Create the `.oav/` directory tree under `work_dir` and extract embedded assets.
pub fn ensure_oav_dirs(work_dir: &Path) -> Result<()> {
    oav_lib::scaffold::ensure_dirs(work_dir, OAV_DIRS)?;

    let compose_path = work_dir.join(".oav/docker-compose.yaml");
    if !compose_path.exists() {
        fs::write(&compose_path, DOCKER_COMPOSE_YAML)
            .with_context(|| format!("failed to write {}", compose_path.display()))?;
    }

    Ok(())
}

/// Ensure `.oav/` is in `.gitignore` under the `# openapi-validator` header.
///
/// Creates `.gitignore` if it doesn't exist. Appends the entry if missing.
pub fn manage_gitignore(work_dir: &Path) -> Result<()> {
    oav_lib::scaffold::ensure_workspace_gitignore(work_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_oav_dirs_creates_all() {
        let tmp = tempfile::tempdir().unwrap();
        ensure_oav_dirs(tmp.path()).unwrap();
        for dir in OAV_DIRS {
            assert!(tmp.path().join(dir).is_dir(), "{dir} not created");
        }
    }

    #[test]
    fn ensure_oav_dirs_writes_compose_file() {
        let tmp = tempfile::tempdir().unwrap();
        ensure_oav_dirs(tmp.path()).unwrap();
        let compose = tmp.path().join(".oav/docker-compose.yaml");
        assert!(compose.exists());
        let content = fs::read_to_string(&compose).unwrap();
        assert!(content.contains("build-spring"));
        assert!(content.contains("build-client-typescript-axios"));
    }

    #[test]
    fn ensure_oav_dirs_preserves_existing_compose() {
        let tmp = tempfile::tempdir().unwrap();
        let compose = tmp.path().join(".oav/docker-compose.yaml");
        fs::create_dir_all(compose.parent().unwrap()).unwrap();
        fs::write(&compose, "custom content").unwrap();

        ensure_oav_dirs(tmp.path()).unwrap();

        let content = fs::read_to_string(&compose).unwrap();
        assert_eq!(content, "custom content");
    }

    #[test]
    fn manage_gitignore_appends_oav_entry() {
        let tmp = tempfile::tempdir().unwrap();
        let gi = tmp.path().join(".gitignore");
        fs::write(&gi, "node_modules/\n").unwrap();

        manage_gitignore(tmp.path()).unwrap();

        let content = fs::read_to_string(&gi).unwrap();
        assert!(content.contains(".oav/"), "workspace entry missing");
        assert!(content.contains("# openapi-validator"), "header missing");
    }

    #[test]
    fn manage_gitignore_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let gi = tmp.path().join(".gitignore");
        fs::write(&gi, ".oav/\n").unwrap();

        manage_gitignore(tmp.path()).unwrap();

        let content = fs::read_to_string(&gi).unwrap();
        // Should not duplicate entries.
        assert_eq!(content.matches(".oav/").count(), 1);
    }

    #[test]
    fn manage_gitignore_creates_when_missing() {
        let tmp = tempfile::tempdir().unwrap();
        manage_gitignore(tmp.path()).unwrap();
        let content = fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
        assert!(content.contains(".oav/"));
        assert!(content.contains("# openapi-validator"));
    }
}
