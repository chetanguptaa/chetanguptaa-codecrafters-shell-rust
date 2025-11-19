use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

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
            return Ok(());
        }
        None => {
            writeln!(err_handle, "{cmd}: command not found")?;
            return Ok(());
        }
    }
}

pub fn run_pipeline(
    shell: &mut Shell,
    cmd: &str,
    args: &[&str],
    redirect_out: Option<&str>,
    redirect_err: Option<&str>,
    pipeline: Vec<Vec<&str>>,
) -> ShellResult<()> {
    let mut out_handle = builtins::get_output_stream(redirect_out)?;
    let mut err_handle = builtins::get_output_stream(redirect_err)?;
    if shell.resolve_command(cmd).is_none() {
        writeln!(err_handle, "{cmd}: command not found")?;
        return Ok(());
    }
    let mut stages: Vec<(String, Vec<String>)> = Vec::new();
    stages.push((
        cmd.to_string(),
        args.iter().map(|s| s.to_string()).collect(),
    ));
    for stage in &pipeline {
        if stage.is_empty() {
            continue;
        }
        let prog = stage[0].to_string();
        let prog_args = stage[1..].iter().map(|s| s.to_string()).collect();
        stages.push((prog, prog_args));
    }
    let mut children = Vec::new();
    let mut prev_stdout: Option<std::process::ChildStdout> = None;
    for (i, (prog, prog_args)) in stages.iter().enumerate() {
        let is_last = i == stages.len() - 1;
        let mut cmd = Command::new(prog);
        cmd.args(prog_args);
        if let Some(stdin_src) = prev_stdout.take() {
            cmd.stdin(Stdio::from(stdin_src));
        } else {
            cmd.stdin(Stdio::inherit());
        }
        if is_last {
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());
        } else {
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
        }
        let mut child = cmd.spawn()?;
        if !is_last {
            prev_stdout = Some(
                child
                    .stdout
                    .take()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "child has no stdout"))?,
            );
        }
        children.push(child);
    }
    let last = children.pop().unwrap();
    let output = last.wait_with_output()?;
    out_handle.write_all(&output.stdout)?;
    err_handle.write_all(&output.stderr)?;
    for mut child in children {
        let _ = child.wait();
    }
    Ok(())
}
