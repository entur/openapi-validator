use anyhow::{Context, Result, bail};
use include_dir::{Dir, DirEntry};
use std::fs;
use std::path::Path;

pub use oav_lib::scaffold::{
    GITIGNORE_HEADER, OAV_DIR, add_gitignore_entries, ensure_workspace_gitignore,
    remove_gitignore_entries,
};

pub fn ensure_oav_dir(root: &Path) -> Result<()> {
    oav_lib::scaffold::ensure_dirs(root, &[OAV_DIR])
}

pub fn prepare_runtime_dirs(root: &Path) -> Result<()> {
    oav_lib::scaffold::ensure_dirs(
        root,
        &[
            ".oav/reports/lint",
            ".oav/reports/generate/server",
            ".oav/reports/generate/client",
            ".oav/reports/compile/server",
            ".oav/reports/compile/client",
            ".oav/generated",
        ],
    )?;
    fs::write(root.join(".oav/status.tsv"), "").context("Failed to create .oav/status.tsv")?;
    Ok(())
}

pub fn extract_assets(root: &Path, assets: &Dir) -> Result<()> {
    let target = root.join(OAV_DIR);
    fs::create_dir_all(&target).context("Failed to create .oav directory")?;
    write_assets(&target, assets)?;
    Ok(())
}

fn write_assets(target: &Path, dir: &Dir) -> Result<()> {
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(child) => {
                let dest = target.join(child.path());
                fs::create_dir_all(&dest)
                    .with_context(|| format!("Failed to create {}", dest.display()))?;
                write_assets(target, child)?;
            }
            DirEntry::File(file) => {
                let dest = target.join(file.path());
                if dest.exists() {
                    if dest.is_file() {
                        continue;
                    }
                    bail!(
                        "Cannot write asset {}: path exists but is not a regular file",
                        dest.display()
                    );
                }
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create {}", parent.display()))?;
                }
                fs::write(&dest, file.contents())
                    .with_context(|| format!("Failed to write {}", dest.display()))?;
                set_script_permissions(&dest)?;
            }
        }
    }
    Ok(())
}

fn set_script_permissions(path: &Path) -> Result<()> {
    if path.extension().and_then(|ext| ext.to_str()) != Some("sh") {
        return Ok(());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perm = fs::Permissions::from_mode(0o755);
        fs::set_permissions(path, perm)
            .with_context(|| format!("Failed to set permissions on {}", path.display()))?;
    }
    Ok(())
}

// Spec discovery

pub use oav_lib::spec::looks_like_openapi;
pub use oav_lib::spec::normalize_spec_path;

pub fn discover_spec(root: &Path, quiet: bool, max_depth: usize) -> Result<Option<String>> {
    let matches = oav_lib::spec::discover_spec(root, max_depth)?;
    if matches.is_empty() {
        return Ok(None);
    }
    select_spec_from_candidates(matches, quiet)
}

fn select_spec_from_candidates(candidates: Vec<String>, quiet: bool) -> Result<Option<String>> {
    if candidates.len() == 1 {
        return Ok(Some(candidates.into_iter().next().unwrap()));
    }

    if quiet {
        bail!(
            "Multiple OpenAPI specs found but interactive selection is disabled in quiet mode. \
             Pass --spec explicitly."
        );
    }

    let theme = dialoguer::theme::ColorfulTheme::default();
    let selection = dialoguer::Select::with_theme(&theme)
        .with_prompt("Multiple specs found — select one")
        .items(&candidates)
        .default(0)
        .interact_on_opt(&console::Term::stderr())?;

    Ok(selection.map(|i| candidates[i].clone()))
}

// Asset diff detection

/// Compare on-disk generator configs against embedded defaults.
/// Returns relative paths (e.g. `generators/server/spring.yaml`) for any files
/// that differ from the embedded version or exist on disk but not in embedded assets.
pub fn find_modified_generator_configs(root: &Path, assets: &Dir) -> Vec<String> {
    let oav_dir = root.join(OAV_DIR);
    let generators_dir = oav_dir.join("generators");
    if !generators_dir.exists() {
        return Vec::new();
    }

    let mut modified = Vec::new();
    let mut embedded_paths = std::collections::HashSet::new();

    // Check embedded generator files against on-disk versions
    collect_embedded_generator_diffs(&oav_dir, assets, &mut modified, &mut embedded_paths);

    // Find on-disk files with no embedded counterpart (user-created configs)
    if let Ok(walker) = fs::read_dir(&generators_dir) {
        collect_extra_files(&oav_dir, walker, &embedded_paths, &mut modified);
    }

    modified.sort();
    modified
}

fn collect_embedded_generator_diffs(
    oav_dir: &Path,
    dir: &Dir,
    modified: &mut Vec<String>,
    embedded_paths: &mut std::collections::HashSet<String>,
) {
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(child) => {
                collect_embedded_generator_diffs(oav_dir, child, modified, embedded_paths);
            }
            DirEntry::File(file) => {
                let rel = file.path().to_string_lossy().to_string();
                if !rel.starts_with("generators/") {
                    continue;
                }
                embedded_paths.insert(rel.clone());
                let disk_path = oav_dir.join(file.path());
                if let Ok(disk_content) = fs::read(&disk_path)
                    && disk_content != file.contents()
                {
                    modified.push(rel);
                }
            }
        }
    }
}

fn collect_extra_files(
    oav_dir: &Path,
    entries: fs::ReadDir,
    embedded_paths: &std::collections::HashSet<String>,
    modified: &mut Vec<String>,
) {
    for dir_entry in entries.filter_map(Result::ok) {
        let ft = match dir_entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        if ft.is_symlink() {
            continue;
        }
        let path = dir_entry.path();
        if ft.is_dir() {
            if let Ok(sub) = fs::read_dir(&path) {
                collect_extra_files(oav_dir, sub, embedded_paths, modified);
            }
        } else if ft.is_file()
            && let Ok(rel) = path.strip_prefix(oav_dir)
        {
            let rel_str = rel.to_string_lossy().to_string();
            if !embedded_paths.contains(&rel_str) {
                modified.push(rel_str);
            }
        }
    }
}
