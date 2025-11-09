use std::env;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

fn main() {
    let built_ins = ["exit", "echo", "type"];
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
            "type" => handle_type(&args, &built_ins),
            _ => handle_external_programs(&args, &cmd),
        }
    }
}

fn handle_type(args: &[&str], built_ins: &[&str]) {
    if args.is_empty() {
        eprintln!("type: missing argument");
        return;
    }
    let target = args[0];
    if built_ins.contains(&target) {
        println!("{target} is a shell builtin");
        return;
    }
    match env::var("PATH") {
        Ok(path) => {
            for dir in path.split(':').filter(|s| !s.is_empty()) {
                let file_path = format!("{dir}/{target}");
                if let Ok(metadata) = fs::metadata(&file_path) {
                    if metadata.permissions().mode() & 0o111 != 0 {
                        println!("{target} is {file_path}");
                        return;
                    }
                }
            }
            println!("{target}: not found");
        }
        Err(_) => println!("PATH variable not set"),
    }
}

fn handle_external_programs(args: &[&str], cmd: &str) {
    match env::var("PATH") {
        Ok(path) => {
            for dir in path.split(':').filter(|s| !s.is_empty()) {
                let file_path = format!("{dir}/{cmd}");
                if let Ok(metadata) = fs::metadata(&file_path) {
                    if metadata.permissions().mode() & 0o111 != 0 {
                        let output = Command::new(cmd)
                            .args(args)
                            .output()
                            .expect("failed to execute process on Unix-like OS");
                        if output.status.success() {
                            io::stdout().write_all(&output.stdout).unwrap();
                        }
                        return;
                    }
                }
            }
            println!("{cmd}: command not found");
        }
        Err(_) => println!("PATH variable not set"),
    }
}
