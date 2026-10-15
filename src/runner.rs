use std::process::Command;

use anyhow::{Context, Result};

/// Runs the command and returns its exit code.
pub fn run_target_command(run_cmd: &str, cwd: Option<&str>) -> Result<i32> {
    let mut command = Command::new("sh");

    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }

    let status = command
        .arg("-c")
        .arg(run_cmd)
        .status()
        .with_context(|| format!("failed to run command: {run_cmd}"))?;

    Ok(status.code().unwrap_or(1))
}
