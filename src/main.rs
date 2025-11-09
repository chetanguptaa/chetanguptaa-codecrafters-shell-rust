mod builtins;
mod error;
mod exec;
mod shell;

use crate::shell::Shell;

fn main() {
    let mut shell = Shell::new();
    if let Err(err) = shell.run() {
        eprintln!("Shell exited with error: {err}");
    }
}
