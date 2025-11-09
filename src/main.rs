#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
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
        }
        if command.starts_with("echo") {
            let args = &command[4..].trim();
            println!("{}", args);
            continue;
        }
        println!("{command}: command not found");
    }
}
