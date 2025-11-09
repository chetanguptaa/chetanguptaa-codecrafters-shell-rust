mod shell;
mod error;
mod builtins;
mod exec;

use crate::shell::Shell;

fn main() {
    let mut shell = Shell::new();
    if let Err(err) = shell.run() {
        eprintln!("Shell exited with error: {err}");
    }
}
