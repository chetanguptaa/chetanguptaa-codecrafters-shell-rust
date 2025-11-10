use std::collections::{HashMap, HashSet};
use std::io::{self, Write};
use std::thread::current;

use crate::builtins;
use crate::error::ShellResult;
use crate::exec;

pub struct Shell {
    pub builtins: HashSet<String>,
    path_cache: HashMap<String, std::path::PathBuf>,
    running: bool,
}

enum QuoteState {
    None,
    InSingle,
    InDouble,
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
    fn handle_input(&mut self, input: &str) -> ShellResult<()> {
        let parts: Vec<String> = Self::parse_args(input);
        if parts.is_empty() {
            return Ok(());
        }
        let cmd = &parts[0];
        let args: Vec<&str> = parts.iter().skip(1).map(|s| s.as_str()).collect();
        match cmd.as_str() {
            "exit" => self.running = false,
            "echo" => builtins::echo(&args)?,
            "type" => builtins::r#type(self, &args)?,
            "pwd" => builtins::pwd()?,
            "cd" => builtins::cd(&args)?,
            "cat" => builtins::cat(&args)?,
            _ => exec::run_external(self, cmd, &args)?,
        }
        Ok(())
    }
    fn parse_args(input: &str) -> Vec<String> {
        let mut args = Vec::new();
        let mut current_arg = String::new();
        let mut state = QuoteState::None;
        for c in input.chars() {
            match state {
                QuoteState::None => {
                    match c {
                        '\'' => state = QuoteState::InSingle,
                        '"' => state = QuoteState::InDouble,
                        c if c.is_whitespace() => {
                            if !current_arg.is_empty() {
                                args.push(current_arg);
                                current_arg = String::new();
                            } else {
                                current_arg.push(c);
                            }
                        }
                        '\\' => {
                            continue;
                        } 
                        _ => {
                            current_arg.push(c);
                        }
                    }
                }
                QuoteState::InSingle => {
                    match c {
                        '\'' => state = QuoteState::None, 
                        _ => current_arg.push(c),
                    }
                }
                QuoteState::InDouble => {
                    match c {
                        '"' => state = QuoteState::None, 
                        _ => current_arg.push(c),
                    }
                }
            } 
        }
        if !current_arg.is_empty() {
            args.push(current_arg);
        }
        args
    }
}
