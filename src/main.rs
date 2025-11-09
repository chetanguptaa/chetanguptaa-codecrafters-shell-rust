#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    print!("$ ");
    let mut command = String::new();
    io::stdin()
        .read_line(&mut command)
        .expect("Failed to read line");
    command = command.trim().to_string();
    print!("$ {command} command not found");
    io::stdout().flush().unwrap();
}
