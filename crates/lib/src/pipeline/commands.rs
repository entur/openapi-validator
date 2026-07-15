use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, bail};

use crate::config::{Config, Mode};
use crate::custom::{CompileBlock, CustomGeneratorDef};
use crate::docker::{self, ContainerCommand};
use crate::generators;

const OAV_DIR: &str = ".oav";

#[derive(Debug, Clone)]
pub struct DockerStep {
    pub cmd: ContainerCommand,
    pub log_path: PathBuf,
    pub command_line: String,
}

pub fn to_posix_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub fn container_path(root: &Path, path: &Path) -> Result<String> {
    let rel = if path.is_absolute() {
        path.strip_prefix(root).with_context(|| {
            format!(
                "path {} is outside repository {}",
                path.display(),
                root.display()
            )
        })?
    } else {
        path
    };
    ensure_safe_relative(rel)?;
    Ok(format!("/work/{}", to_posix_path(rel)))
}

fn ensure_safe_relative(path: &Path) -> Result<()> {
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        bail!("path {} resolves outside repository", path.display());
    }
    Ok(())
}

fn timeout(cfg: &Config) -> Duration {
    Duration::from_secs(cfg.docker_timeout)
}

fn command_line(args: &[String]) -> String {
    format!("$ docker {}", shell_words::join(args))
}

pub fn spectral_command(cfg: &Config, root: &Path, spec_path: &Path) -> Result<DockerStep> {
    let spec = container_path(root, spec_path)?;
    let mut args = vec![
        "run".into(),
        "--rm".into(),
        "-v".into(),
        format!("{}:/work", root.display()),
    ];
    args.extend(docker::user_args());
    args.extend([
        cfg.spectral_image.clone(),
        "lint".into(),
        spec,
        "--ruleset".into(),
        cfg.spectral_ruleset.clone(),
        "--fail-severity".into(),
        cfg.spectral_fail_severity.clone(),
        "-f".into(),
        "stylish".into(),
    ]);
    Ok(step(cfg, root.join(".oav/reports/lint/spectral.log"), args))
}

pub fn redocly_command(cfg: &Config, root: &Path, spec_path: &Path) -> Result<DockerStep> {
    let spec = container_path(root, spec_path)?;
    let mut args = vec![
        "run".into(),
        "--rm".into(),
        "-v".into(),
        format!("{}:/work", root.display()),
    ];
    args.extend(docker::user_args());
    args.extend([
        "-w".into(),
        "/work".into(),
        cfg.redocly_image.clone(),
        "lint".into(),
        spec,
        "--format".into(),
        "stylish".into(),
    ]);
    Ok(step(cfg, root.join(".oav/reports/lint/redocly.log"), args))
}

pub fn builtin_generate_command(
    cfg: &Config,
    root: &Path,
    spec_path: &Path,
    generator: &str,
    scope: &str,
    config_path: Option<&str>,
) -> Result<DockerStep> {
    let spec = container_path(root, spec_path)?;
    let output_dir = format!("/work/.oav/generated/{scope}/{generator}");

    let mut args = vec![
        "run".into(),
        "--rm".into(),
        "-v".into(),
        format!("{}:/work", root.display()),
    ];
    args.extend(docker::user_args());
    args.extend([
        cfg.generator_image.clone(),
        "generate".into(),
        "-i".into(),
        spec,
        "-g".into(),
        generator.to_string(),
        "-o".into(),
        output_dir,
    ]);

    if let Some(path) = config_path {
        args.extend(["-c".into(), path.to_string()]);
    }

    Ok(step(
        cfg,
        root.join(format!(".oav/reports/generate/{scope}/{generator}.log")),
        args,
    ))
}

pub fn builtin_generate_from_config_command(
    cfg: &Config,
    root: &Path,
    spec_path: &Path,
    generator: &str,
    scope: &str,
    config_path: &Path,
) -> Result<DockerStep> {
    let spec = container_path(root, spec_path)?;
    let config = container_path(root, config_path)?;

    let mut args = vec!["run".into(), "--rm".into()];
    args.extend(docker::user_args());
    args.extend([
        "-v".into(),
        format!("{}:/work", root.display()),
        "-w".into(),
        format!("/work/{OAV_DIR}"),
        cfg.generator_image.clone(),
        "generate".into(),
        "-i".into(),
        spec,
        "-c".into(),
        config,
    ]);

    Ok(step(
        cfg,
        root.join(format!(".oav/reports/generate/{scope}/{generator}.log")),
        args,
    ))
}

pub fn builtin_compile_command(
    cfg: &Config,
    root: &Path,
    generator: &str,
    scope: &str,
) -> Result<DockerStep> {
    let service = compile_service_name(generator, scope);
    let compose_file = root.join(crate::scaffold::DOCKER_COMPOSE_FILE);
    let project_dir = root.join(".oav");

    let args = vec![
        "compose".into(),
        "-f".into(),
        compose_file.display().to_string(),
        "--project-directory".into(),
        project_dir.display().to_string(),
        "run".into(),
        "--rm".into(),
        service,
    ];

    Ok(step(
        cfg,
        root.join(format!(".oav/reports/compile/{scope}/{generator}.log")),
        args,
    ))
}

pub fn compile_service_name(generator: &str, scope: &str) -> String {
    match scope {
        "server" => format!("build-{generator}"),
        _ => format!("build-client-{generator}"),
    }
}

pub fn custom_generate_command(
    cfg: &Config,
    root: &Path,
    spec_path: &Path,
    def: &CustomGeneratorDef,
) -> Result<DockerStep> {
    let spec = container_path(root, spec_path)?;
    let resolved_command = def.generate.command.replace("{spec}", &spec);
    let cmd_args = shell_words::split(&resolved_command)
        .with_context(|| format!("Failed to parse generate command for '{}'", def.name))?;

    let mut args = vec![
        "run".into(),
        "--rm".into(),
        "-v".into(),
        format!("{}:/work", root.display()),
    ];
    args.extend(docker::user_args());
    args.push(def.generate.image.clone());
    args.extend(cmd_args);

    Ok(step(
        cfg,
        root.join(format!(
            ".oav/reports/generate/{}/{}.log",
            def.scope, def.name
        )),
        args,
    ))
}

pub fn custom_compile_command(
    cfg: &Config,
    root: &Path,
    def: &CustomGeneratorDef,
    compile: &CompileBlock,
) -> Result<DockerStep> {
    let workdir = format!("/work/.oav/generated/{}/{}", def.scope, def.name);
    let cmd_args = shell_words::split(&compile.command)
        .with_context(|| format!("Failed to parse compile command for '{}'", def.name))?;

    let mut args = vec![
        "run".into(),
        "--rm".into(),
        "-v".into(),
        format!("{}:/work", root.display()),
        "-w".into(),
        workdir,
        compile.image.clone(),
    ];
    args.extend(cmd_args);

    Ok(step(
        cfg,
        root.join(format!(
            ".oav/reports/compile/{}/{}.log",
            def.scope, def.name
        )),
        args,
    ))
}

pub fn resolve_config_path(
    cfg: &Config,
    root: &Path,
    generator: &str,
    scope: &str,
) -> Result<Option<String>> {
    if let Some(user_path) = cfg.generator_overrides.get(generator) {
        return Ok(Some(container_path(
            root,
            &resolve_under_root(root, user_path)?,
        )?));
    }
    if generators::find_builtin(generator, scope).is_some() {
        return Ok(Some(format!("/work/.oav/configs/{scope}/{generator}.yaml")));
    }
    Ok(None)
}

fn resolve_under_root(root: &Path, path: &str) -> Result<PathBuf> {
    let candidate = Path::new(path);
    let resolved = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        root.join(candidate)
    };
    if !resolved.starts_with(root) {
        bail!("config path {} resolves outside repository", path);
    }
    Ok(resolved)
}

pub fn build_generator_list(
    cfg: &Config,
    custom_defs: &[CustomGeneratorDef],
) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let mut add_for_scope = |generators: &[String], scope: &str, custom: &[CustomGeneratorDef]| {
        if generators.is_empty() {
            for def in generators::builtin_generators_for_scope(scope) {
                pairs.push((def.name.to_string(), scope.to_string()));
            }
            for def in custom.iter().filter(|d| d.scope == scope) {
                pairs.push((def.name.clone(), scope.to_string()));
            }
        } else {
            for generator in generators {
                pairs.push((generator.clone(), scope.to_string()));
            }
        }
    };

    match cfg.mode {
        Mode::Server => add_for_scope(&cfg.server_generators, "server", custom_defs),
        Mode::Client => add_for_scope(&cfg.client_generators, "client", custom_defs),
        Mode::Both => {
            add_for_scope(&cfg.server_generators, "server", custom_defs);
            add_for_scope(&cfg.client_generators, "client", custom_defs);
        }
    }

    pairs
}

pub fn write_builtin_configs(
    cfg: &Config,
    root: &Path,
    generators: &[(String, String)],
) -> Result<()> {
    for (name, scope) in generators {
        if cfg.generator_overrides.contains_key(name.as_str()) {
            continue;
        }
        if let Some(def) = generators::find_builtin(name, scope) {
            let config_dir = root.join(format!(".oav/configs/{scope}"));
            std::fs::create_dir_all(&config_dir).with_context(|| {
                format!("failed to create config directory {}", config_dir.display())
            })?;
            let config_path = config_dir.join(format!("{name}.yaml"));
            std::fs::write(&config_path, def.config_yaml).with_context(|| {
                format!("failed to write config file {}", config_path.display())
            })?;
        }
    }
    Ok(())
}

fn step(cfg: &Config, log_path: PathBuf, args: Vec<String>) -> DockerStep {
    DockerStep {
        command_line: command_line(&args),
        cmd: ContainerCommand {
            args,
            timeout: timeout(cfg),
        },
        log_path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Mode;

    fn test_config() -> Config {
        Config {
            spectral_image: "stoplight/spectral:6".into(),
            spectral_ruleset: "https://example.com/.spectral.yml".into(),
            spectral_fail_severity: "error".into(),
            redocly_image: "redocly/cli:1.25.5".into(),
            generator_image: "openapitools/openapi-generator-cli:v7.17.0".into(),
            docker_timeout: 120,
            mode: Mode::Both,
            server_generators: vec!["spring".into()],
            client_generators: vec!["typescript-axios".into()],
            ..Config::default()
        }
    }

    fn custom_def(name: &str, scope: &str) -> CustomGeneratorDef {
        CustomGeneratorDef {
            name: name.into(),
            scope: scope.into(),
            generate: crate::custom::GenerateBlock {
                image: "my-image:latest".into(),
                command: "gen --spec {spec} --output /work/.oav/generated/server/my-gen".into(),
            },
            compile: Some(crate::custom::CompileBlock {
                image: "build-image:latest".into(),
                command: "npm run build".into(),
            }),
        }
    }

    #[test]
    fn nested_spec_paths_are_root_relative() {
        let root = Path::new("/tmp/repo");
        let spec = root.join("apis/petstore/openapi.yaml");
        assert_eq!(
            container_path(root, &spec).unwrap(),
            "/work/apis/petstore/openapi.yaml"
        );
    }

    #[test]
    fn spectral_command_uses_nested_spec_path() {
        let cfg = test_config();
        let cmd = spectral_command(
            &cfg,
            Path::new("/tmp/repo"),
            Path::new("/tmp/repo/apis/spec.yaml"),
        )
        .unwrap();
        assert!(cmd.cmd.args.contains(&"/work/apis/spec.yaml".into()));
    }

    #[test]
    fn redocly_command_sets_workdir() {
        let cfg = test_config();
        let cmd = redocly_command(&cfg, Path::new("/tmp"), Path::new("/tmp/spec.yaml")).unwrap();
        let w_pos = cmd.cmd.args.iter().position(|a| a == "-w").unwrap();
        assert_eq!(cmd.cmd.args[w_pos + 1], "/work");
    }

    #[test]
    fn generate_commands_include_user_args() {
        let cfg = test_config();
        let cmd = builtin_generate_command(
            &cfg,
            Path::new("/tmp"),
            Path::new("/tmp/spec.yaml"),
            "spring",
            "server",
            None,
        )
        .unwrap();
        assert!(cmd.cmd.args.contains(&"--user".into()) || docker::user_args().is_empty());
    }

    #[test]
    fn builtin_generate_from_config_builds_log_and_args() {
        let cfg = test_config();
        let cmd = builtin_generate_from_config_command(
            &cfg,
            Path::new("/tmp"),
            Path::new("/tmp/spec.yaml"),
            "spring",
            "server",
            Path::new("/tmp/.oav/generators/server/spring.yaml"),
        )
        .unwrap();
        assert!(cmd.cmd.args.contains(&"-c".into()));
        assert!(
            cmd.cmd
                .args
                .contains(&"/work/.oav/generators/server/spring.yaml".into())
        );
        assert_eq!(
            cmd.log_path,
            Path::new("/tmp/.oav/reports/generate/server/spring.log")
        );
    }

    #[test]
    fn custom_generate_interpolates_nested_spec() {
        let cfg = test_config();
        let def = custom_def("my-gen", "server");
        let cmd = custom_generate_command(
            &cfg,
            Path::new("/tmp/repo"),
            Path::new("/tmp/repo/apis/spec.yaml"),
            &def,
        )
        .unwrap();
        assert!(cmd.cmd.args.iter().any(|a| a == "/work/apis/spec.yaml"));
        assert!(!cmd.cmd.args.iter().any(|a| a.contains("{spec}")));
    }

    #[test]
    fn compile_commands_omit_user_args() {
        let cfg = test_config();
        let builtin = builtin_compile_command(&cfg, Path::new("/tmp"), "spring", "server").unwrap();
        let def = custom_def("my-gen", "server");
        let custom =
            custom_compile_command(&cfg, Path::new("/tmp"), &def, def.compile.as_ref().unwrap())
                .unwrap();
        assert!(!builtin.cmd.args.contains(&"--user".into()));
        assert!(!custom.cmd.args.contains(&"--user".into()));
    }

    #[test]
    fn compile_service_names_match_scope() {
        assert_eq!(compile_service_name("spring", "server"), "build-spring");
        assert_eq!(
            compile_service_name("typescript-axios", "client"),
            "build-client-typescript-axios"
        );
    }

    #[test]
    fn build_generator_list_empty_defaults_to_builtins() {
        let mut cfg = test_config();
        cfg.server_generators.clear();
        cfg.client_generators.clear();
        let pairs = build_generator_list(&cfg, &[]);
        assert_eq!(pairs.len(), 14);
    }

    #[test]
    fn resolve_config_path_builtin_and_override() {
        let mut cfg = test_config();
        assert_eq!(
            resolve_config_path(&cfg, Path::new("/tmp"), "spring", "server")
                .unwrap()
                .as_deref(),
            Some("/work/.oav/configs/server/spring.yaml")
        );
        cfg.generator_overrides
            .insert("spring".into(), ".oav/custom/spring.yaml".into());
        assert_eq!(
            resolve_config_path(&cfg, Path::new("/tmp"), "spring", "server")
                .unwrap()
                .as_deref(),
            Some("/work/.oav/custom/spring.yaml")
        );
    }

    #[test]
    fn write_builtin_configs_creates_files() {
        let tmp = tempfile::tempdir().unwrap();
        let cfg = test_config();
        write_builtin_configs(&cfg, tmp.path(), &[("spring".into(), "server".into())]).unwrap();
        let content =
            std::fs::read_to_string(tmp.path().join(".oav/configs/server/spring.yaml")).unwrap();
        assert!(content.contains("generatorName: spring"));
    }
}
