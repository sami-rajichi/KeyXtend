//! The command line: nothing or `--show` shows the keyboard, and `--quit` ends a running copy. Anything else is refused
//! before Qt starts, so no Qt option (such as a plugin path) reaches the uiAccess process.

use kx_platform::ui::Command;
use kx_platform_windows::instance::{flag, from_flag};
use std::ffi::OsString;
use std::fmt;

/// The command when no argument is given: start, or show the running copy.
const DEFAULT: Command = Command::Show;
/// Every command a flag can name, for the usage line.
const COMMANDS: [Command; 2] = [Command::Show, Command::Quit];

/// Why the command line was refused.
#[derive(Debug, PartialEq, Eq)]
pub enum ArgError {
    /// An argument that names no command, as the user wrote it.
    Unknown(String),
    /// More than one argument.
    Many,
}

impl fmt::Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let flags: Vec<String> = COMMANDS.into_iter().map(flag).collect();
        let flags = flags.join(" or ");
        match self {
            Self::Unknown(arg) => write!(f, "unknown argument {arg:?}; use {flags}"),
            Self::Many => write!(f, "give one argument at most: {flags}"),
        }
    }
}

impl std::error::Error for ArgError {}

/// The command `args` ask for, without the program's own name; nothing means Show.
///
/// # Errors
/// `Unknown` for an argument that is not a flag, `Many` for more than one argument.
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Command, ArgError> {
    let mut args = args.into_iter();
    let Some(first) = args.next() else {
        return Ok(DEFAULT);
    };
    if args.next().is_some() {
        return Err(ArgError::Many);
    }
    first
        .to_str()
        .and_then(from_flag)
        .ok_or_else(|| ArgError::Unknown(first.to_string_lossy().into_owned()))
}

#[cfg(test)]
mod tests;
