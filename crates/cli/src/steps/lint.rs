use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::cli::Linter;
use crate::config::Config;
use crate::docker;
use crate::output::Output;
use crate::util::{OAV_DIR, append_status, write_log_header};

pub fn run(root: &Path, spec_path: &Path, config: &Config, output: &Output) -> Result<bool> {
    match config.linter {
        Linter::Spectral => run_linter(
            root,
            output,
            oav_lib::pipeline::spectral_command(config, root, spec_path)?,
            "spectral",
        ),
        Linter::Redocly => run_linter(
            root,
            output,
            oav_lib::pipeline::redocly_command(config, root, spec_path)?,
            "redocly",
        ),
        Linter::None => Ok(true),
    }
}

fn run_linter(
    root: &Path,
    output: &Output,
    step: oav_lib::pipeline::DockerStep,
    linter: &str,
) -> Result<bool> {
    let reports_dir = root.join(OAV_DIR).join("reports").join("lint");
    fs::create_dir_all(&reports_dir).context("Failed to create lint reports directory")?;
    write_log_header(&step.log_path, &step.command_line)?;
    let success =
        docker::run_with_logging(step.cmd.args, &step.log_path, output, step.cmd.timeout)?;
    append_status(
        root,
        "lint",
        "spec",
        linter,
        if success { "ok" } else { "fail" },
        &step.log_path,
    )?;
    Ok(success)
}
