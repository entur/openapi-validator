use anyhow::{Context, Result, bail};
use include_dir::{Dir, DirEntry};
use std::fs;
use std::path::Path;

pub use oav_lib::scaffold::{
    GITIGNORE_HEADER, OAV_DIR, add_gitignore_entries, ensure_workspace_gitignore,
    prepare_workspace, remove_gitignore_entries,
};

pub fn prepare_runtime_dirs(root: &Path) -> Result<()> {
    oav_lib::scaffold::prepare_workspace(root)?;
    fs::write(root.join(".oav/status.tsv"), "").context("Failed to create .oav/status.tsv")?;
    Ok(())
}

pub fn extract_assets(root: &Path, assets: &Dir) -> Result<()> {
    let target = root.join(OAV_DIR);
    fs::create_dir_all(&target).context("Failed to create .oav directory")?;
    write_assets(&target, assets)?;
    write_generator_references(&target)?;
    Ok(())
}

/// Write the embedded default generator configs to `.oav/generators/{scope}/{name}.yaml`
/// as editable references. Existing files are left untouched.
fn write_generator_references(target: &Path) -> Result<()> {
    for def in builtin_generator_defs() {
        let dir = target.join("generators").join(def.scope);
        fs::create_dir_all(&dir).with_context(|| format!("Failed to create {}", dir.display()))?;
        let dest = dir.join(format!("{}.yaml", def.name));
        if !dest.exists() {
            fs::write(&dest, def.config_yaml)
                .with_context(|| format!("Failed to write {}", dest.display()))?;
        }
    }
    Ok(())
}

fn builtin_generator_defs() -> impl Iterator<Item = &'static oav_lib::generators::GeneratorDef> {
    oav_lib::generators::builtin_server_generators()
        .iter()
        .chain(oav_lib::generators::builtin_client_generators())
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
pub fn find_modified_generator_configs(root: &Path) -> Vec<String> {
    let oav_dir = root.join(OAV_DIR);
    let generators_dir = oav_dir.join("generators");
    if !generators_dir.exists() {
        return Vec::new();
    }

    let mut modified = Vec::new();
    let mut embedded_paths = std::collections::HashSet::new();

    // Check embedded generator configs against on-disk versions
    for def in builtin_generator_defs() {
        let rel = format!("generators/{}/{}.yaml", def.scope, def.name);
        embedded_paths.insert(rel.clone());
        if let Ok(disk_content) = fs::read(oav_dir.join(&rel))
            && disk_content != def.config_yaml.as_bytes()
        {
            modified.push(rel);
        }
    }

    // Find on-disk files with no embedded counterpart (user-created configs)
    if let Ok(walker) = fs::read_dir(&generators_dir) {
        collect_extra_files(&oav_dir, walker, &embedded_paths, &mut modified);
    }

    modified.sort();
    modified
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
