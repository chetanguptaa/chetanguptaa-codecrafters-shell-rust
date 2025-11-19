#![allow(dead_code)]

use nix::errno::Errno;
use std::io;

pub type ShellResult<T> = Result<T, ShellError>;

#[derive(Debug)]
pub enum ShellError {
    Io(io::Error),
    InvalidInput(String),
    ExecError(String),
}

impl std::fmt::Display for ShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellError::Io(e) => write!(f, "IO error: {e}"),
            ShellError::InvalidInput(s) => write!(f, "{s}"),
            ShellError::ExecError(s) => write!(f, "{s}"),
        }
    }
}

impl From<io::Error> for ShellError {
    fn from(err: io::Error) -> Self {
        ShellError::Io(err)
    }
}

impl From<Errno> for ShellError {
    fn from(err: Errno) -> Self {
        ShellError::Io(io::Error::from_raw_os_error(err as i32))
    }
}
