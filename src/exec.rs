use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use crate::builtins;
use crate::error::{ShellResult};
use crate::shell::Shell;

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
    redirect_err: Option<&str>,
) -> ShellResult<()> {
    let mut out_handle = builtins::get_output_stream(redirect_out)?;
    let mut err_handle = builtins::get_output_stream(redirect_err)?;

    match shell.resolve_command(cmd) {
        Some(_) => {
            let output = Command::new(cmd).args(args).output()?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            write!(out_handle, "{}", stdout)?;
            if !output.status.success() {
                if redirect_out.is_some() {
                    write!(err_handle, "{}", stderr)?;
                } else {
                    write!(err_handle, "{}", stderr)?;
                }
            }
            Ok(())
        }
        None => {
            writeln!(err_handle, "{cmd}: command not found")?;
            Ok(())
        }
    }
}
