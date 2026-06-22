use anyhow::{Context, Result, bail};
use std::fs::OpenOptions;
use std::io::{self, Write as IoWrite};
use std::path::Path;
use std::time::Duration;

use oav_lib::docker::{CancelToken, ContainerCommand, OutputLine, spawn};

use crate::output::Output;

pub use oav_lib::docker::{ensure_available, user_args, user_flag};

/// Run a docker command, streaming output to the log file (appended) and to
/// stdout/stderr when `output.verbose` is true. Returns `Ok(success)` for
/// normal completions; bails on spawn failure or timeout.
pub fn run_with_logging(
    args: Vec<String>,
    log_path: &Path,
    output: &Output,
    timeout: Duration,
) -> Result<bool> {
    run_inner(args, log_path, timeout, output.verbose)
}

/// Like [`run_with_logging`] but never tees to stdout/stderr.
pub fn run_with_logging_quiet(
    args: Vec<String>,
    log_path: &Path,
    timeout: Duration,
) -> Result<bool> {
    run_inner(args, log_path, timeout, false)
}

fn run_inner(args: Vec<String>, log_path: &Path, timeout: Duration, tee: bool) -> Result<bool> {
    let cmd = ContainerCommand { args, timeout };
    let rx = spawn(cmd, CancelToken::new()).context("Failed to start Docker command")?;

    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .with_context(|| format!("Failed to open log file {}", log_path.display()))?;

    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    for line in rx {
        match line {
            OutputLine::Stdout(s) => {
                writeln!(log, "{s}").context("Failed to write to log file")?;
                if tee {
                    writeln!(stdout, "{s}").ok();
                }
            }
            OutputLine::Stderr(s) => {
                writeln!(log, "{s}").context("Failed to write to log file")?;
                if tee {
                    writeln!(stderr, "{s}").ok();
                }
            }
            OutputLine::Done(result) => {
                if result.timed_out {
                    bail!("Docker command timed out after {}s", timeout.as_secs());
                }
                return Ok(result.success);
            }
        }
    }

    // Channel closed without a Done frame — treat as failure.
    Ok(false)
}
