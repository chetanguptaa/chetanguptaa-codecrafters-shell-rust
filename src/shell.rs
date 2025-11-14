use std::collections::{HashMap, HashSet};
use std::io::{self, Write};

use crate::builtins;
use crate::error::ShellResult;
use crate::exec;

pub struct Shell {
    pub builtins: HashSet<String>,
    path_cache: HashMap<String, std::path::PathBuf>,
    running: bool,
}

#[derive(PartialEq)]
enum QuoteState {
    None,
    InSingle,
    InDouble,
}

impl Shell {
    pub fn new() -> Self {
        let builtins = ["exit", "echo", "type", "pwd", "cd", "cat"]
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
            if let Err(e) = self.handle_input(input) {
                eprintln!("shell: error: {}", e);
            }
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
        let mut args: Vec<&str> = Vec::new();
        let mut redirect_out: Option<&str> = None;
        let mut i = 1;
        while i < parts.len() {
            match parts[i].as_str() {
                ">" | "1>" => {
                    if i + 1 < parts.len() {
                        if redirect_out.is_some() {
                            eprint!("shell: error: multiple output redirects");
                            return Ok(());
                        }
                        redirect_out = Some(&parts[i + 1]);
                        i += 2;
                    } else {
                        eprint!("shell: error: missing filename after redirection");
                        return Ok(());
                    }
                }
                _ => {
                    args.push(&parts[i]);
                    i += 1;
                }
            }
        }
        match cmd.as_str() {
            "exit" => self.running = false,
            "echo" => builtins::echo(&args, redirect_out)?,
            "type" => builtins::r#type(self, &args, redirect_out)?,
            "pwd" => builtins::pwd(redirect_out)?,
            "cd" => builtins::cd(&args)?,
            "cat" => builtins::cat(&args, redirect_out)?,
            _ => exec::run_external(self, cmd, &args, redirect_out)?,
        }
        Ok(())
    }
    fn parse_args(input: &str) -> Vec<String> {
        let mut args = Vec::new();
        let mut current_arg = String::new();
        let mut state = QuoteState::None;
        let mut last_was_escape = false;
        let mut in_argument = false;
        for c in input.chars() {
            match state {
                QuoteState::None => {
                    if last_was_escape {
                        current_arg.push(c);
                        last_was_escape = false;
                        in_argument = true;
                    } else {
                        match c {
                            '\\' => last_was_escape = true,
                            '\'' => {
                                state = QuoteState::InSingle;
                                in_argument = true;
                            }
                            '"' => {
                                state = QuoteState::InDouble;
                                in_argument = true;
                            }
                            c if c.is_whitespace() => {
                                if in_argument {
                                    args.push(std::mem::take(&mut current_arg));
                                    in_argument = false;
                                }
                            }
                            _ => {
                                current_arg.push(c);
                                in_argument = true;
                            }
                        }
                    }
                }
                QuoteState::InSingle => match c {
                    '\'' => state = QuoteState::None,
                    _ => current_arg.push(c),
                },
                QuoteState::InDouble => {
                    if last_was_escape {
                        match c {
                            '"' | '\\' => current_arg.push(c),
                            _ => {
                                current_arg.push('\\');
                                current_arg.push(c);
                            }
                        }
                        last_was_escape = false;
                    } else {
                        match c {
                            '\\' => last_was_escape = true,
                            '"' => state = QuoteState::None,
                            _ => current_arg.push(c),
                        }
                    }
                }
            }
        }
        if last_was_escape {
            current_arg.push('\\');
            in_argument = true;
        }
        if in_argument {
            args.push(current_arg);
        }
        args
    }
}
