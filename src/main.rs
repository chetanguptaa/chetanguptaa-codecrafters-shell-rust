use std::env;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    const BUILT_INS: [&str; 5] = ["exit", "echo", "type", "pwd", "cd"];
    loop {
        if let Err(e) = run_shell(&BUILT_INS) {
            eprintln!("Error: {}", e);
        }
    }
}

fn run_shell(built_ins: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    print!("$ ");
    io::stdout().flush()?;
    let mut input = String::new();
    if io::stdin().read_line(&mut input)? == 0 {
        std::process::exit(0);
    }
    let input = input.trim();
    if input.is_empty() {
        return Ok(());
    }
    let mut parts = input.split_whitespace();
    let cmd = parts.next().unwrap();
    let args: Vec<&str> = parts.collect();
    match cmd {
        "exit" => std::process::exit(0),
        "echo" => println!("{}", args.join(" ")),
        "type" => handle_type(&args, built_ins)?,
        "pwd" => handle_pwd()?,
        "cd" => handle_cd(&args)?,
        _ => handle_external_command(cmd, &args)?,
    }

    Ok(())
}

fn handle_type(args: &[&str], built_ins: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let Some(target) = args.first() else {
        eprintln!("type: missing argument");
        return Ok(());
    };
    if built_ins.contains(target) {
        println!("{target} is a shell builtin");
        return Ok(());
    }
    match find_executable(target) {
        Some(path) => println!("{target} is {}", path.display()),
        None => println!("{target}: not found"),
    }
    Ok(())
}

fn handle_external_command(cmd: &str, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let Some(exec_path) = find_executable(cmd) else {
        println!("{cmd}: command not found");
        return Ok(());
    };
    let output = Command::new(exec_path).args(args).output();
    match output {
        Ok(output) => {
            if output.status.success() {
                io::stdout().write_all(&output.stdout)?;
            } else {
                io::stderr().write_all(&output.stderr)?;
            }
        }
        Err(err) => eprintln!("Failed to execute {cmd}: {err}"),
    }
    Ok(())
}

fn handle_pwd() -> Result<(), Box<dyn std::error::Error>> {
    match env::current_dir() {
        Ok(path) => println!("{}", path.display()),
        Err(err) => eprintln!("Error: failed to get current directory: {}", err),
    }
    Ok(())
}

fn handle_cd(args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let Some(target) = args.first() else {
        eprintln!("cd: missing argument");
        return Ok(());
    };
    let new_dir = Path::new(target);
    if let Err(err) = env::set_current_dir(&new_dir) {
        eprintln!("cd: {}: {}", new_dir.display(), err);
    }
    Ok(())
}

fn find_executable(name: &str) -> Option<PathBuf> {
    let path_var = env::var("PATH").ok()?;
    for dir in path_var.split(':').filter(|s| !s.is_empty()) {
        let path = PathBuf::from(dir).join(name);
        if let Ok(meta) = fs::metadata(&path) {
            if meta.is_file() && meta.permissions().mode() & 0o111 != 0 {
                return Some(path);
            }
        }
    }
    None
}
