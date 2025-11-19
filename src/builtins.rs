use crate::error::{ShellError, ShellResult};
use crate::shell::Shell;
use std::env;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

pub fn get_output_stream(redirect_out: Option<&str>) -> ShellResult<Box<dyn Write>> {
    match redirect_out {
        Some(filename) => {
            let path = Path::new(filename);
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .write(true)
                .open(path)?;
            Ok(Box::new(file))
        }
        None => Ok(Box::new(io::stdout())),
    }
}

pub fn echo(
    args: &[&str],
    redirect_stdout: Option<&str>,
    redirect_stderr: Option<&str>,
) -> ShellResult<()> {
    let mut out_handle = get_output_stream(redirect_stdout)?;
    let mut err_handle = get_output_stream(redirect_stderr)?;
    if args.is_empty() {
        writeln!(out_handle, "")?;
    } else {
        let output = args.join(" ");
        writeln!(out_handle, "{}", output)?;
    }
    out_handle.flush()?;
    err_handle.flush()?;
    Ok(())
}

pub fn pwd(redirect_stdout: Option<&str>, redirect_stderr: Option<&str>) -> ShellResult<()> {
    let mut out_handle = get_output_stream(redirect_stdout)?;
    let mut err_handle = get_output_stream(redirect_stderr)?;
    let dir = env::current_dir()?;
    writeln!(out_handle, "{}", dir.display())?;
    out_handle.flush()?;
    err_handle.flush()?;
    Ok(())
}

pub fn cd(args: &[&str]) -> ShellResult<()> {
    if args.is_empty() {
        return Err(ShellError::InvalidInput("cd: missing argument".into()));
    }
    let target = if args[0] == "~" {
        dirs::home_dir().ok_or_else(|| ShellError::InvalidInput("No home dir".into()))?
    } else {
        Path::new(args[0]).to_path_buf()
    };
    if let Err(_) = env::set_current_dir(&target) {
        println!("cd: {}: No such file or directory", target.display());
    }
    Ok(())
}

pub fn cmd_type(
    shell: &mut Shell,
    args: &[&str],
    redirect_stdout: Option<&str>,
    redirect_stderr: Option<&str>,
) -> ShellResult<()> {
    let Some(name) = args.first() else {
        return Err(ShellError::InvalidInput("type: missing argument".into()));
    };
    let mut out_handle = get_output_stream(redirect_stdout)?;
    let mut err_handle = get_output_stream(redirect_stderr)?;
    if shell.builtins.contains(*name) {
        writeln!(out_handle, "{name} is a shell builtin")?;
        return Ok(());
    }
    match shell.resolve_command(name) {
        Some(path) => {
            writeln!(out_handle, "{name} is {}", path.display())?;
        }
        None => writeln!(err_handle, "{}: not found", name)?,
    }
    out_handle.flush()?;
    err_handle.flush()?;
    Ok(())
}

pub fn history(
    shell: &mut Shell,
    args: &[&str],
    redirect_stdout: Option<&str>,
    redirect_stderr: Option<&str>,
) -> ShellResult<()> {
    let mut out_handle = get_output_stream(redirect_stdout)?;
    let mut err_handle = get_output_stream(redirect_stderr)?;
    if args.len() > 0 {
        if (args[0] == "-r" || args[0] == "-w" || args[0] == "-a") && args.len() <= 1 {
            return Err(ShellError::InvalidInput("history: missing argument".into()));
        } else if args[0] == "-r" && args.len() > 1 {
            let history_file = args[1];
            if history_file.is_empty() {
                return Err(ShellError::InvalidInput("history: missing argument".into()));
            }
            let content = std::fs::read_to_string(history_file);
            match content {
                Ok(data) => {
                    for (_, line) in data.lines().enumerate() {
                        shell.history.push(line.to_string());
                    }
                }
                Err(e) => {
                    writeln!(err_handle, "history: {}: {}", history_file, e)?;
                }
            }
            shell.history_file_index = shell.history.len();
        } else if args[0] == "-w" && args.len() > 1 {
            let history_file = args[1];
            if history_file.is_empty() {
                return Err(ShellError::InvalidInput("history: missing argument".into()));
            }
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(history_file)?;
            for command in &shell.history {
                writeln!(file, "{}", command)?;
            }
            shell.history_file_index = shell.history.len();
        } else if args[0] == "-a" && args.len() > 1 {
            let history_file = args[1];
            if history_file.is_empty() {
                return Err(ShellError::InvalidInput("history: missing argument".into()));
            }
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(history_file)?;
            let start = shell.history_file_index;
            let end = shell.history.len();
            for cmd in &shell.history[start..end] {
                writeln!(file, "{}", cmd)?;
            }
            shell.history_file_index = end;

        }
        else {
            let mut i = args.len() - 1;
            while i < args.len() {
                let arg = args[i];
                match arg.parse::<usize>() {
                    Ok(n) => {
                        let start = if n > shell.history.len() {
                            0
                        } else {
                            shell.history.len() - n
                        };
                        for (index, command) in shell.history[start..].iter().enumerate() {
                            writeln!(out_handle, "    {} {}", start + index + 1, command)?;
                        }
                    }
                    Err(_) => {
                        writeln!(err_handle, "history: {}: invalid number", arg)?;
                    }
                }
                i -= 1;
            }
        }
    } else {
        for (index, command) in shell.history.iter().enumerate() {
            writeln!(out_handle, "    {} {}", index + 1, command)?;
        }
    }

    out_handle.flush()?;
    err_handle.flush()?;
    Ok(())
}

pub fn exit(shell: &mut Shell) -> ShellResult<()> {
    if let Ok(file_path) = env::var("HISTFILE") {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true) 
            .truncate(true)
            .open(file_path)?;

        for cmd in &shell.history {
            writeln!(file, "{}", cmd)?;
        }
    }
    shell.running = false;
    Ok(())
}