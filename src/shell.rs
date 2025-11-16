use std::collections::{HashMap, HashSet};
use std::io::{self, Stdout, Write};

use crate::builtins;
use crate::error::ShellResult;
use crate::exec;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

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
            let stdout = io::stdout();
            let mut stdout = stdout.into_raw_mode()?;
            stdout.flush()?;
            let mut input = String::new();
            let mut first_tab = false;
            let mut common_prefix_exists = false;
            for key in io::stdin().keys() {
                match key? {
                    Key::Char('\n') => {
                        write!(stdout, "\r\n")?;
                        stdout.flush()?;
                        drop(stdout);
                        let trimmed = input.trim();
                        if trimmed.is_empty() {
                            break;
                        }
                        if let Err(e) = self.handle_input(trimmed) {
                            eprintln!("shell: error: {}", e);
                        }
                        break;
                    }
                    Key::Char('\t') => {
                        if input.ends_with(' ') {
                            input.push_str("    ");
                            Self::redraw_line(&mut stdout, &input)?;
                            first_tab = false;
                            common_prefix_exists = false;
                            continue;
                        }
                        let parts = Self::parse_args(&input);
                        let (matches, last) = if let Some(last) = parts.last() {
                            (self.find_completions(last), last)
                        } else {
                            print!("\x07");
                            first_tab = false;
                            common_prefix_exists = false;
                            stdout.flush()?;
                            continue;
                        };
                        if matches.is_empty() {
                            print!("\x07");
                            first_tab = false;
                            common_prefix_exists = false;
                            stdout.flush()?;
                            continue;
                        }
                        if matches.len() == 1 {
                            let completion = &matches[0][last.len()..];
                            input.push_str(completion);
                            input.push(' ');
                            Self::redraw_line(&mut stdout, &input)?;
                            first_tab = false;
                            common_prefix_exists = false;
                            continue;
                        }
                        let lcp = Self::longest_common_prefix(&matches);
                        let has_new_lcp = lcp.len() > last.len();
                        if !first_tab {
                            first_tab = true;
                            if has_new_lcp {
                                common_prefix_exists = true;
                                let completion = &lcp[last.len()..];
                                input.push_str(completion);
                                Self::redraw_line(&mut stdout, &input)?;
                                first_tab = false;
                            } else {
                                common_prefix_exists = false;
                                print!("\x07");
                                stdout.flush()?;
                            }
                        } else {
                            if !common_prefix_exists {
                                write!(stdout, "\r\n")?;
                                for m in matches {
                                    print!("{}  ", m); 
                                }
                                write!(stdout, "\r\n")?;
                                Self::redraw_line(&mut stdout, &input)?;
                            } else {
                                print!("\x07");
                                stdout.flush()?;
                            }
                            first_tab = false;
                            common_prefix_exists = false;
                        }
                    }
                    Key::Char(c) => {
                        first_tab = false;
                        common_prefix_exists = false;
                        input.push(c);
                        Self::redraw_line(&mut stdout, &input)?;
                    }
                    Key::Backspace => {
                        first_tab = false;
                        common_prefix_exists = false;
                        input.pop();
                        Self::redraw_line(&mut stdout, &input)?;
                    }
                    _ => {
                        first_tab = false;
                        common_prefix_exists = false;
                        self.running = false;
                        break;
                    }
                }
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
        let mut redirect_stdout: Option<&str> = None;
        let mut redirect_stderr: Option<&str> = None;
        let mut pipeline_input: Option<&[String]> = None;
        let mut i = 1;
        while i < parts.len() {
            match parts[i].as_str() {
                ">" | "1>" => {
                    if redirect_stdout.is_some() {
                        eprintln!("shell: error: multiple stdout redirects");
                        return Ok(());
                    }
                    if i + 1 >= parts.len() {
                        eprintln!("shell: error: missing filename after redirection");
                        return Ok(());
                    }
                    redirect_stdout = Some(&parts[i + 1]);
                    i += 2;
                }
                "2>" => {
                    if redirect_stderr.is_some() {
                        eprintln!("shell: error: multiple stderr redirects");
                        return Ok(());
                    }
                    if i + 1 >= parts.len() {
                        eprintln!("shell: error: missing filename after redirection");
                        return Ok(());
                    }
                    redirect_stderr = Some(&parts[i + 1]);
                    i += 2;
                }
                ">>" | "1>>" => {
                    if redirect_stdout.is_some() {
                        eprintln!("shell: error: multiple stdout redirects");
                        return Ok(());
                    }
                    if i + 1 >= parts.len() {
                        eprintln!("shell: error: missing filename after redirection");
                        return Ok(());
                    }
                    redirect_stdout = Some(&parts[i + 1]);
                    i += 2;
                }
                "2>>" => {
                    if redirect_stderr.is_some() {
                        eprintln!("shell: error: multiple stderr redirects");
                        return Ok(());
                    }
                    if i + 1 >= parts.len() {
                        eprintln!("shell: error: missing filename after redirection");
                        return Ok(());
                    }
                    redirect_stderr = Some(&parts[i + 1]);
                    i += 2;
                }
                "|" => {
                    if pipeline_input.is_some() {
                        eprintln!("shell: error: multiple pipeline input");
                        return Ok(()); 
                    }
                    if i + 1 >= parts.len() {
                        eprintln!("shell: error: missing new cmd after pipeline");
                        return Ok(());
                    }
                    pipeline_input = Some(&parts[i + 1 ..]);
                    i = parts.len();
                }
                _ => {
                    args.push(&parts[i]);
                    i += 1;
                }
            }
        }
        match cmd.as_str() {
            "exit" => self.running = false,
            "echo" => builtins::echo(&args, redirect_stdout, redirect_stderr)?,
            "type" => builtins::r#type(self, &args, redirect_stdout, redirect_stderr)?,
            "pwd" => builtins::pwd(redirect_stdout, redirect_stderr)?,
            "cd" => builtins::cd(&args)?,
            _ => exec::run_external(self, cmd, &args, redirect_stdout, redirect_stderr, pipeline_input)?,
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

    fn redraw_line(stdout: &mut RawTerminal<Stdout>, input: &str) -> ShellResult<()>{
        write!(
            stdout,
            "\r{}{}",
            termion::clear::CurrentLine,
            format!("$ {}", input)
        )?;
        stdout.flush()?;
        Ok(())
    }

    fn find_completions(&self, prefix: &str) -> Vec<String> {
        let mut matches: Vec<String> = self
            .builtins
            .iter()
            .filter(|b| b.starts_with(prefix))
            .cloned()
            .collect();
        if let Some(dir) = std::env::var_os("PATH") {
            for path in std::env::split_paths(&dir) {
                if let Ok(entries) = std::fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let file_name = entry.file_name();
                        let file_name_str = file_name.to_string_lossy();
                        if file_name_str.starts_with(prefix) {
                            matches.push(file_name_str.to_string());
                        }
                    }
                }
            }
        }
        matches.sort();
        matches.dedup();
        matches
    }

    fn longest_common_prefix(strs: &[String]) -> String {
        if strs.is_empty() {
            return String::new();
        }
        let first_string = &strs[0];
        let mut common_prefix = String::new();
        for (i, char_from_first) in first_string.chars().enumerate() {
            for s in strs.iter().skip(1) {
                match s.chars().nth(i) {
                    Some(s_char) => {
                        if s_char != char_from_first {
                            return common_prefix;
                        }
                    }
                    None => {
                        return common_prefix;
                    }
                }
            }
            common_prefix.push(char_from_first);
        }
        common_prefix
    }
}
