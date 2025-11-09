use std::env;
use std::path::Path;

use crate::error::{ShellError, ShellResult};
use crate::shell::Shell;

pub fn echo(args: &[&str]) -> ShellResult<()> {
    println!("{}", args.join(" "));
    Ok(())
}

pub fn pwd() -> ShellResult<()> {
    let dir = env::current_dir()?;
    println!("{}", dir.display());
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
    if let Err(e) = env::set_current_dir(&target) {
        println!("cd: {}: No such file or directory", target.display());
    }
    Ok(())
}

pub fn r#type(shell: &mut Shell, args: &[&str]) -> ShellResult<()> {
    let Some(name) = args.first() else {
        return Err(ShellError::InvalidInput("type: missing argument".into()));
    };
    if shell.builtins.contains(*name) {
        println!("{name} is a shell builtin");
        return Ok(());
    }
    match shell.resolve_command(name) {
        Some(path) => println!("{name} is {}", path.display()),
        None => println!("{name}: not found"),
    }
    Ok(())
}
