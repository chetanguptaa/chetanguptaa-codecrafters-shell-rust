use crate::error::{ShellError, ShellResult};
use crate::shell::Shell;
use std::env;
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Write};
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
                .write(true)
                .truncate(true)
                .open(path)?;
            Ok(Box::new(BufWriter::new(file)))
        }
        None => Ok(Box::new(io::stdout())),
    }
}

pub fn echo(args: &[&str], redirect_stdout: Option<&str>, redirect_stderr: Option<&str>) -> ShellResult<()> {
    let mut out_handle = get_output_stream(redirect_stdout)?;
    let _err_handle = get_output_stream(redirect_stderr)?;
    if args.is_empty() {
        writeln!(out_handle, "")?;
    } else {
        let output = args.join(" ");
        writeln!(out_handle, "{}", output)?;
    }
    Ok(())
}

pub fn pwd(redirect_stdout: Option<&str>, redirect_stderr: Option<&str>) -> ShellResult<()> {
    let mut out_handle = get_output_stream(redirect_stdout)?;
    let _err_handle = get_output_stream(redirect_stderr)?;
    let dir = env::current_dir()?;
    writeln!(out_handle, "{}", dir.display())?;
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

pub fn r#type(shell: &mut Shell, args: &[&str], redirect_stdout: Option<&str>, redirect_stderr: Option<&str>) -> ShellResult<()> {
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
    Ok(())
}