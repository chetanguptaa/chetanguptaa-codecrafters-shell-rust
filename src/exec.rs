use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

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

pub fn run_external(shell: &mut Shell, cmd: &str, args: &[&str]) -> ShellResult<()> {
    match shell.resolve_command(cmd) {
        Some(path) => {
            let output = Command::new(path).args(args).output()?;
            if output.status.success() {
                print!("{}", String::from_utf8_lossy(&output.stdout));
            } else {
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
            }
            Ok(())
        }
        None => {
            println!("{cmd}: command not found");
            Ok(())
        }
    }
}
