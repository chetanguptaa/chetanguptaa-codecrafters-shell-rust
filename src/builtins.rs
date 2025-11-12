use crate::error::{ShellError, ShellResult};
use crate::shell::Shell;
use std::env;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

pub fn get_output_stream(redirect_out: Option<&str>) -> ShellResult<Box<dyn Write>> {
    match redirect_out {
        Some(filename) => {
            let file = File::create(filename)?;
            Ok(Box::new(BufWriter::new(file)))
        }
        None => Ok(Box::new(io::stdout())),
    }
}

pub fn echo(args: &[&str], redirect_out: Option<&str>) -> ShellResult<()> {
    let mut handle = get_output_stream(redirect_out)?;
    if args.is_empty() {
        write!(handle, "")?;
    } else {
        let output = args.join(" ");
        write!(handle, "{}", output)?;
    }
    Ok(())
}

pub fn pwd(redirect_out: Option<&str>) -> ShellResult<()> {
    let mut handle = get_output_stream(redirect_out)?;
    let dir = env::current_dir()?;
    writeln!(handle, "{}", dir.display().to_string().trim_end())?;
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

pub fn r#type(shell: &mut Shell, args: &[&str], redirect_out: Option<&str>) -> ShellResult<()> {
    let Some(name) = args.first() else {
        return Err(ShellError::InvalidInput("type: missing argument".into()));
    };
    let mut handle = get_output_stream(redirect_out)?;
    if shell.builtins.contains(*name) {
        writeln!(handle, "{name} is a shell builtin")?;
        return Ok(());
    }
    match shell.resolve_command(name) {
        Some(path) => writeln!(handle, "{name} is {}", path.display())?,
        None => writeln!(handle, "{name}: not found")?,
    }
    Ok(())
}

pub fn cat(args: &[&str], redirect_out: Option<&str>) -> ShellResult<()> {
    if args.is_empty() {
        return Err(ShellError::InvalidInput("cat: missing argument".into()));
    }
    let mut handle = get_output_stream(redirect_out)?;
    for filename in args {
        let content = std::fs::read_to_string(filename);
        match content {
            Ok(text) => write!(handle, "{}", text)?,
            Err(_) => writeln!(handle, "cat: {}: No such file or directory", filename)?,
        }
    }
    Ok(())
}