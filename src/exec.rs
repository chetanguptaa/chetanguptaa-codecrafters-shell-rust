use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use crate::error::{ShellError, ShellResult};
use crate::shell::Shell;
use crate::builtins;

pub fn find_executable(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var("PATH").ok()?;
    for dir in path_var.split(':') {
        let path = PathBuf::from(dir).join(name);
        if let Ok(meta) = fs::metadata(&path) {
            if meta.permissions().mode() & 0o111 != 0 {
                return Some(path);
            }
        }
    }
    None
}

pub fn run_external(
    shell: &mut Shell,
    cmd: &str,
    args: &[&str],
    redirect_out: Option<&str>,
) -> ShellResult<()> {
    // This gets the file (e.g., quz.md) or stdout
    let mut handle = builtins::get_output_stream(redirect_out)?;

    match shell.resolve_command(cmd) {
        Some(_) => {
            // Run the command and capture all its output
            let output = Command::new(cmd).args(args).output()?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            write!(handle, "{}", stdout)?;
            eprint!("{}", stderr);
            if !output.status.success() {
                return Err(ShellError::InvalidInput(format!(
                    "{cmd}: command failed"
                )));
            }
            Ok(())
        }
        None => {
            eprintln!("{cmd}: command not found");
            Ok(())
        }
    }
}
