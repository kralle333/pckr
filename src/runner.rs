use std::process::Command;

pub fn run_target_command(run_cmd: &str, cwd: Option<String>) {
    let mut command = Command::new("sh");

    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }

    let status = command.arg("-c").arg(run_cmd).status();
    match status {
        Ok(status) if status.success() => {}
        Ok(status) => eprintln!("Command exited with non-zero code: {status}"),
        Err(err) => eprintln!("Failed to run command: {err}"),
    }
}
