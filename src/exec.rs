use nix::libc;
use nix::sys::wait::waitpid;
use nix::unistd::{close, dup2, fork, pipe, ForkResult};
use std::fs::{self, OpenOptions};
use std::os::fd::RawFd;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;

use crate::builtins;
use crate::error::{ShellError, ShellResult};
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

pub fn run_pipeline(
    shell: &mut Shell,
    cmd: &str,
    args: &[&str],
    redirect_out: Option<&str>,
    redirect_err: Option<&str>,
    pipeline: Vec<Vec<&str>>,
) -> ShellResult<()> {
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
    let num_stages = stages.len();
    let mut pipes: Vec<(RawFd, RawFd)> = Vec::new();
    for _ in 0..num_stages - 1 {
        pipes.push(pipe()?);
    }
    let mut pids = Vec::new();
    for i in 0..num_stages {
        let is_first = i == 0;
        let is_last = i == num_stages - 1;
        let (ref prog, ref prog_args) = stages[i];
        match unsafe { fork() } {
            Ok(ForkResult::Child) => {
                unsafe {
                    libc::signal(libc::SIGPIPE, libc::SIG_DFL);
                }
                if !is_first {
                    let (read_fd, _) = pipes[i - 1];
                    dup2(read_fd, 0)?;
                }
                if !is_last {
                    let (_, write_fd) = pipes[i];
                    dup2(write_fd, 1)?;
                } else {
                    if let Some(outfile) = redirect_out {
                        let fd = OpenOptions::new()
                            .create(true)
                            .append(true)
                            .write(true)
                            .open(outfile)?
                            .as_raw_fd();
                        dup2(fd, 1)?;
                    }
                    if let Some(errfile) = redirect_err {
                        let fd = OpenOptions::new()
                            .create(true)
                            .append(true)
                            .write(true)
                            .open(errfile)?
                            .as_raw_fd();
                        dup2(fd, 2)?;
                    }
                }
                for (read_fd, write_fd) in &pipes {
                    close(*read_fd).ok();
                    close(*write_fd).ok();
                }
                if shell.builtins.contains(prog.as_str()) {
                    let args_refs: Vec<&str> = prog_args.iter().map(|s| s.as_str()).collect();
                    let result: Result<(), ShellError> = match prog.as_str() {
                        "echo" => builtins::echo(&args_refs, None, None),
                        "pwd" => builtins::pwd(None, None),
                        "type" => builtins::cmd_type(shell, &args_refs, None, None),
                        "cd" => builtins::cd(&args_refs),
                        _ => {
                            eprintln!("builtin '{}' not implemented", prog);
                            std::process::exit(1);
                        }
                    };
                    if result.is_ok() { std::process::exit(0); }
                    else { std::process::exit(1); } 
                } else {
                    let c_prog = std::ffi::CString::new(prog.as_str()).unwrap();
                    let mut c_args = Vec::new();
                    c_args.push(c_prog.clone());
                    for a in prog_args {
                        c_args.push(std::ffi::CString::new(a.as_str()).unwrap());
                    }
                    nix::unistd::execvp(&c_prog, &c_args).ok();
                    eprintln!("{}: command not found", prog);
                    std::process::exit(127);
                }
            }
            Ok(ForkResult::Parent { child }) => {
                pids.push(child);
            }
            Err(e) => {
                return Err(ShellError::from(e));
            }
        }
    }
    for (r, w) in pipes {
        let _ = close(r);
        let _ = close(w);
    }
    for pid in pids {
        let _ = waitpid(pid, None);
    }
    Ok(())
}
