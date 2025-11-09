#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    let built_in_command = vec!["exit", "echo", "type"];
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read line");
        let command = command.trim();
        if command.starts_with("exit") {
            break;
        } else if command.starts_with("echo") {
            let args = &command[4..].trim();
            println!("{}", args);
            continue;
        } else if command.starts_with("type") {
            let args = &command[4..].trim();
            if built_in_command.contains(&args) {
                println!("{args} is a shell builtin");
            } else {
                println!("{args}: not found");
            }
        } else {
            println!("{command}: command not found");
        }
    }
}
