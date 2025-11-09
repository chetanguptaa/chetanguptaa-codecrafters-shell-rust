use std::env;
use std::fs;
use std::io::{self, Write};

fn main() {
    let built_ins = vec!["exit", "echo", "type"];
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
            _ => println!("{cmd}: command not found"),
        }
    }
}

fn handle_type(args: &Vec<&str>, built_ins: &Vec<&str>) {
    if args.is_empty() {
        eprintln!("type: missing argument");
        return;
    }
    let target = args[0];
    if built_ins.contains(&target) {
        println!("{target} is a shell builtin");
        return;
    }
    if let Ok(path) = env::var("PATH") {
        for dir in path.split(':').filter(|s| !s.is_empty()) {
            let file_path = format!("{}/{}", dir, target);
            if fs::metadata(&file_path).is_ok() {
                println!("{target} is {}", file_path);
                return;
            }
        }
        println!("{target}: not found");
    } else {
        println!("PATH variable not set");
    }
}
