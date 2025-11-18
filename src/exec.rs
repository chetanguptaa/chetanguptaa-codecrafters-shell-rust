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
    pipeline: Option<&[String]>,
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
    if let Some(rest) = pipeline {
        if !rest.is_empty() {
            let prog = rest[0].clone();
            let prog_args = rest[1..].to_vec();
            stages.push((prog, prog_args));
        }
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
        if is_last && pipeline.is_some() {
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
