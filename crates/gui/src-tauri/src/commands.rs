use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;
use tauri::State;
use tauri::ipc::Channel;

use oav_lib::config::Config;
use oav_lib::custom::CustomGeneratorDef;
use oav_lib::fix::FixProposal;
use oav_lib::generators::GeneratorDef;
use oav_lib::log_parser::LintError;
use oav_lib::pipeline::{PipelineEvent, PipelineInput, ValidateReport, run_pipeline};
use oav_lib::trace::CompileError;
use oav_lib::{config, custom, docker, fetch, fix, generators, log_parser, scaffold, spec, trace};

/// Holds the cancel token of the currently running pipeline, if any.
#[derive(Default)]
pub struct PipelineState(pub Mutex<Option<docker::CancelToken>>);

type CmdResult<T> = Result<T, String>;

fn err_str(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// Resolve a workspace-relative path, rejecting anything that escapes `root`.
fn workspace_path(root: &str, rel: &str) -> CmdResult<PathBuf> {
    let root = Path::new(root)
        .canonicalize()
        .map_err(|e| format!("Invalid workspace root: {e}"))?;
    let joined = root.join(rel);
    // Canonicalize the deepest existing ancestor so `..` segments can't escape
    // the root even for paths that don't exist yet.
    let mut probe = joined.clone();
    while !probe.exists() {
        probe = match probe.parent() {
            Some(p) => p.to_path_buf(),
            None => return Err("Path escapes the workspace".to_string()),
        };
    }
    let resolved = probe.canonicalize().map_err(err_str)?;
    if !resolved.starts_with(&root) {
        return Err("Path escapes the workspace".to_string());
    }
    Ok(joined)
}

fn load_custom_defs(root: &Path, cfg: &Config) -> Vec<CustomGeneratorDef> {
    match &cfg.custom_generators_dir {
        Some(dir) => custom::load(root, dir).unwrap_or_default(),
        None => Vec::new(),
    }
}

#[derive(Serialize)]
pub struct Catalog {
    pub builtin: Vec<&'static GeneratorDef>,
    pub custom: Vec<CustomGeneratorDef>,
}

#[tauri::command]
pub fn discover_specs(root: String) -> CmdResult<Vec<String>> {
    let cfg = config::load(Path::new(&root)).map_err(err_str)?;
    spec::discover_spec(Path::new(&root), cfg.search_depth).map_err(err_str)
}

#[tauri::command]
pub fn load_config(root: String) -> CmdResult<Config> {
    config::load(Path::new(&root)).map_err(err_str)
}

#[tauri::command]
pub fn save_config(root: String, config: Config) -> CmdResult<()> {
    config::write(Path::new(&root), &config).map_err(err_str)
}

/// Non-fatal validation: bails on structural errors, returns warnings otherwise.
#[tauri::command]
pub fn validate_config(root: String, config: Config) -> CmdResult<Vec<String>> {
    let custom_defs = load_custom_defs(Path::new(&root), &config);
    config::validate_for_run(&config, &custom_defs).map_err(err_str)
}

#[tauri::command]
pub fn generator_catalog(root: String) -> CmdResult<Catalog> {
    let cfg = config::load(Path::new(&root)).map_err(err_str)?;
    Ok(Catalog {
        builtin: generators::builtin_generators().collect(),
        custom: load_custom_defs(Path::new(&root), &cfg),
    })
}

/// Ok(()) when Docker (and compose, if `compose` is set) is usable.
#[tauri::command]
pub fn docker_status(compose: bool) -> CmdResult<()> {
    if compose {
        docker::ensure_available_with_compose().map_err(err_str)
    } else {
        docker::ensure_available().map_err(err_str)
    }
}

#[tauri::command]
pub fn read_workspace_file(root: String, path: String) -> CmdResult<String> {
    let full = workspace_path(&root, &path)?;
    fs::read_to_string(&full).map_err(|e| format!("Failed to read {path}: {e}"))
}

#[tauri::command]
pub fn write_workspace_file(root: String, path: String, content: String) -> CmdResult<()> {
    let full = workspace_path(&root, &path)?;
    fs::write(&full, content).map_err(|e| format!("Failed to write {path}: {e}"))
}

/// List files under `.oav/generated/{scope}/{generator}`, relative to root.
#[tauri::command]
pub fn list_generated_files(
    root: String,
    scope: String,
    generator: String,
) -> CmdResult<Vec<String>> {
    let base = Path::new(&root)
        .join(".oav/generated")
        .join(&scope)
        .join(&generator);
    if !base.is_dir() {
        return Ok(Vec::new());
    }
    let root_path = Path::new(&root);
    let mut files: Vec<String> = walkdir::WalkDir::new(&base)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| {
            e.path()
                .strip_prefix(root_path)
                .ok()
                .map(|p| p.to_string_lossy().to_string())
        })
        .collect();
    files.sort();
    Ok(files)
}

#[tauri::command]
pub fn parse_lint(log: String) -> Vec<LintError> {
    log_parser::parse_lint_log(&log)
}

#[tauri::command]
pub fn parse_compile(log: String) -> Vec<CompileError> {
    trace::parse_compile_errors(&log)
}

#[tauri::command]
pub fn load_report(root: String) -> CmdResult<Option<ValidateReport>> {
    let path = Path::new(&root).join(".oav/reports/report.json");
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).map_err(err_str)?;
    serde_json::from_str(&raw).map(Some).map_err(err_str)
}

#[tauri::command]
pub fn propose_fix(
    root: String,
    spec_path: String,
    error: LintError,
) -> CmdResult<Option<FixProposal>> {
    let full = workspace_path(&root, &spec_path)?;
    let raw = fs::read_to_string(&full).map_err(err_str)?;
    let index = spec::parse_spec(&raw).map_err(err_str)?;
    fix::propose_fix(&error, &index, &full).map_err(err_str)
}

#[tauri::command]
pub fn apply_fix(root: String, spec_path: String, proposal: FixProposal) -> CmdResult<()> {
    let full = workspace_path(&root, &spec_path)?;
    fix::apply_fix(&proposal, &full).map_err(err_str)
}

/// Fetch a spec from a URL into `.oav/fetched-spec.{ext}`; returns the
/// root-relative path.
#[tauri::command]
pub fn fetch_spec_url(root: String, url: String) -> CmdResult<String> {
    let root_path = Path::new(&root);
    scaffold::prepare_workspace(root_path).map_err(err_str)?;
    let rel = fetch::fetch_spec(root_path, &url).map_err(err_str)?;
    Ok(rel.to_string_lossy().to_string())
}

/// Start the validation pipeline; events stream over `on_event`.
/// Fails if the config is structurally invalid or Docker is unavailable.
#[tauri::command]
pub fn start_pipeline(
    state: State<'_, PipelineState>,
    root: String,
    spec_path: String,
    on_event: Channel<PipelineEvent>,
) -> CmdResult<()> {
    let root_path = PathBuf::from(&root);
    let cfg = config::load(&root_path).map_err(err_str)?;
    let custom_defs = load_custom_defs(&root_path, &cfg);
    config::validate_for_run(&cfg, &custom_defs).map_err(err_str)?;

    if cfg.compile {
        docker::ensure_available_with_compose().map_err(err_str)?;
    } else {
        docker::ensure_available().map_err(err_str)?;
    }

    scaffold::prepare_workspace(&root_path).map_err(err_str)?;
    if cfg.manage_gitignore {
        let _ = scaffold::ensure_workspace_gitignore(&root_path);
    }

    let spec_abs = workspace_path(&root, &spec_path)?;
    let input = PipelineInput {
        config: cfg,
        custom_defs,
        spec_path: spec_abs,
        work_dir: root_path,
    };

    let cancel = docker::CancelToken::new();
    {
        let mut slot = state.0.lock().map_err(err_str)?;
        if let Some(existing) = slot.take() {
            existing.cancel();
        }
        *slot = Some(cancel.clone());
    }

    let rx = run_pipeline(input, cancel);
    std::thread::spawn(move || {
        for event in rx {
            if on_event.send(event).is_err() {
                break;
            }
        }
    });
    Ok(())
}

#[tauri::command]
pub fn cancel_pipeline(state: State<'_, PipelineState>) -> CmdResult<()> {
    let slot = state.0.lock().map_err(err_str)?;
    if let Some(token) = slot.as_ref() {
        token.cancel();
    }
    Ok(())
}
