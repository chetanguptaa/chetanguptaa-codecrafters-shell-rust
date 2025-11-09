use std::env;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    const BUILT_INS: [&str; 3] = ["exit", "echo", "type"];
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("Failed to read input");
            continue;
        }
        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        let mut parts = input.split_whitespace();
        let cmd = parts.next().unwrap();
        let args: Vec<&str> = parts.collect();
        match cmd {
            "exit" => break,
            "echo" => println!("{}", args.join(" ")),
            "type" => handle_type(&args, &BUILT_INS),
            "pwd" => handle_pwd(),
            _ => handle_external_command(cmd, &args),
        }
    }
}

fn handle_type(args: &[&str], built_ins: &[&str]) {
    let Some(target) = args.first() else {
        eprintln!("type: missing argument");
        return;
    };
    if built_ins.contains(target) {
        println!("{target} is a shell builtin");
        return;
    }
    match find_executable(target) {
        Some(path) => println!("{target} is {}", path.display()),
        None => println!("{target}: not found"),
    }
}

fn handle_external_command(cmd: &str, args: &[&str]) {
    match find_executable(cmd) {
        Some(_) => match Command::new(cmd).args(args).output() {
            Ok(output) => {
                if output.status.success() {
                    io::stdout().write_all(&output.stdout).unwrap();
                } else {
                    io::stderr().write_all(&output.stderr).unwrap();
                }
            }
            Err(err) => eprintln!("Failed to execute {cmd}: {err}"),
        },
        None => println!("{cmd}: command not found"),
    }
}


fn handle_pwd() {
    match env::current_dir() {
        Ok(path) => println!("{}", path.display()),
        Err(err) => {
            eprintln!("Error: failed to get current directory: {}", err);
        }
    }
}

fn find_executable(name: &str) -> Option<PathBuf> {
    let path_var = env::var("PATH").ok()?;
    for dir in path_var.split(':').filter(|s| !s.is_empty()) {
        let path = PathBuf::from(dir).join(name);
        if let Ok(meta) = fs::metadata(&path) {
            if meta.permissions().mode() & 0o111 != 0 {
                return Some(path);
            }
        }
    }
    None
}
