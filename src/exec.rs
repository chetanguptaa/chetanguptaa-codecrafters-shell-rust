use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::Write;

use crate::builtins;
use crate::error::ShellResult;
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
    pipeline_input: Option<&[String]>,
) -> ShellResult<()> {
    let mut out_handle = builtins::get_output_stream(redirect_out)?;
    let mut err_handle = builtins::get_output_stream(redirect_err)?;
    match shell.resolve_command(cmd) {
        None => {
            writeln!(err_handle, "{cmd}: command not found")?;
            return Ok(());
        }
        Some(_) => {}
    }
     let mut first_child = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut previous_stdout = first_child.stdout.take();
    if let Some(pipe_cmd) = pipeline_input {
        if !pipe_cmd.is_empty() {
            let pipe_cmd_name = &pipe_cmd[0];
            let pipe_cmd_args = &pipe_cmd[1..];
            let mut pipe_child = Command::new(pipe_cmd_name)
                .args(pipe_cmd_args)
                .stdin(Stdio::from(previous_stdout.take().unwrap()))
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            _ = pipe_child.stdout.take();
            let output = pipe_child.wait_with_output()?;
            write!(out_handle, "{}", String::from_utf8_lossy(&output.stdout))?;
            write!(err_handle, "{}", String::from_utf8_lossy(&output.stderr))?;
            let _ = first_child.wait();
            return Ok(());
        }
    }
    if let Some(stdout) = previous_stdout {
        use std::io::{BufRead, BufReader};
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let line = line?;
            writeln!(out_handle, "{}", line)?;
        }
    }
    let _ = first_child.wait();
    Ok(()) 
}
