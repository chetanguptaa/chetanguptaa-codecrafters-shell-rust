use std::collections::{HashMap, HashSet};
use std::io::{self, Write};

use crate::error::{ShellResult};
use crate::builtins;
use crate::exec;

pub struct Shell {
    pub builtins: HashSet<String>,
    path_cache: HashMap<String, std::path::PathBuf>,
    running: bool,
}

impl Shell {
    pub fn new() -> Self {
        let builtins = ["exit", "echo", "type", "pwd", "cd"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        Self {
            builtins,
            path_cache: HashMap::new(),
            running: true,
        }
    }
    pub fn run(&mut self) -> ShellResult<()> {
        while self.running {
            print!("$ ");
            io::stdout().flush()?;
            let mut input = String::new();
            if io::stdin().read_line(&mut input)? == 0 {
                break;
            }
            let input = input.trim();
            if input.is_empty() {
                continue;
            }
            self.handle_input(input)?;
        }
        Ok(())
    }
    fn handle_input(&mut self, input: &str) -> ShellResult<()> {
        let mut parts = input.split_whitespace();
        let cmd = match parts.next() {
            Some(c) => c,
            None => return Ok(()),
        };
        let args: Vec<&str> = parts.collect();
        match cmd {
            "exit" => self.running = false,
            "echo" => builtins::echo(&args)?,
            "type" => builtins::r#type(self, &args)?,
            "pwd" => builtins::pwd()?,
            "cd" => builtins::cd(&args)?,
            _ => exec::run_external(self, cmd, &args)?,
        }
        Ok(())
    }
    pub fn resolve_command(&mut self, name: &str) -> Option<std::path::PathBuf> {
        if let Some(p) = self.path_cache.get(name) {
            return Some(p.clone());
        }
        if let Some(p) = exec::find_executable(name) {
            self.path_cache.insert(name.to_string(), p.clone());
            Some(p)
        } else {
            None
        }
    }
}
