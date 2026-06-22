use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

/// Verify that the Docker daemon is reachable.
pub fn ensure_available() -> Result<()> {
    let status = Command::new("docker")
        .arg("version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("failed to invoke `docker` — is it installed and on PATH?")?;

    if !status.success() {
        bail!("Docker is installed but not responding. Is the daemon running?");
    }

    Ok(())
}

/// Verify that the Docker daemon and Compose plugin are reachable.
pub fn ensure_available_with_compose() -> Result<()> {
    ensure_available()?;

    let compose = Command::new("docker")
        .args(["compose", "version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("failed to check Docker Compose — is the Compose plugin installed?")?;

    if !compose.success() {
        bail!(
            "Docker Compose plugin is not available. \
             Update Docker Desktop or install the `docker-compose-plugin` package."
        );
    }

    Ok(())
}

/// Returns `["--user", "uid:gid"]` on Unix so containers write files
/// as the invoking user. Empty on other platforms.
pub fn user_args() -> Vec<String> {
    #[cfg(unix)]
    {
        // SAFETY: geteuid() and getegid() are POSIX getters that always succeed.
        let uid = unsafe { libc::geteuid() };
        let gid = unsafe { libc::getegid() };
        vec!["--user".into(), format!("{uid}:{gid}")]
    }

    #[cfg(not(unix))]
    {
        Vec::new()
    }
}

/// Returns `--user uid:gid` as a single shell-formatted fragment, suitable
/// for log lines that echo the docker command. Empty on non-Unix.
pub fn user_flag() -> String {
    #[cfg(unix)]
    {
        let uid = unsafe { libc::geteuid() };
        let gid = unsafe { libc::getegid() };
        format!("--user {uid}:{gid}")
    }

    #[cfg(not(unix))]
    {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_available_does_not_panic() {
        let _ = ensure_available();
    }

    #[test]
    fn ensure_available_with_compose_does_not_panic() {
        let _ = ensure_available_with_compose();
    }

    #[cfg(unix)]
    #[test]
    fn user_args_returns_pair() {
        let args = user_args();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], "--user");
        assert!(args[1].contains(':'));
    }

    #[cfg(unix)]
    #[test]
    fn user_flag_matches_user_args() {
        let flag = user_flag();
        let args = user_args();
        assert_eq!(flag, format!("{} {}", args[0], args[1]));
    }
}
