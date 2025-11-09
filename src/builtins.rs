use std::env;
use std::path::Path;

use crate::error::{ShellError, ShellResult};
use crate::shell::Shell;

pub fn echo(args: &[&str]) -> ShellResult<()> {
    if args.len() == 1 {
        let arg = args[0];
        if arg.starts_with("'") && arg.ends_with("'") {
            let empty_str: &str = "";
            let new_str= arg.replace('\'', empty_str);
            println!("{}", new_str);
            return Ok(());
        }
        println!("{}", arg);
        return Ok(());
    }
    if args.len() > 1 && args[0].starts_with("'") && args[args.len() - 1].ends_with("'") {
        let first = args[0].trim_start_matches("'");
        let last = args[args.len() - 1].trim_end_matches("'");
        let middle = &args[1..args.len() - 1];
        let mut all_parts = vec![first];
        all_parts.extend_from_slice(middle);
        all_parts.push(last);
        println!("{}", all_parts.join(" "));
        return Ok(());
    }
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
    if let Err(_) = env::set_current_dir(&target) {
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

pub fn cat(args: &[&str]) -> ShellResult<()> {
    if args.is_empty() {
        return Err(ShellError::InvalidInput("cat: missing argument".into()));
    }
    for filename in args {
        let content = std::fs::read_to_string(filename);
        match content {
            Ok(text) => print!("{}", text),
            Err(_) => println!("cat: {}: No such file or directory", filename),
        }
    }
    Ok(())
}
