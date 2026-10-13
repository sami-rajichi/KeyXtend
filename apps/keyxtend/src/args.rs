//! The command line: nothing or `--show` shows the keyboard, and `--quit` ends a running copy. Anything else is refused
//! before Qt starts, so no Qt option (such as a plugin path) reaches the uiAccess process. Only the gate-only build
//! (`gate-input`) also takes `--sim-input`, alone or before `--show`.

use kx_platform::ui::Command;
use kx_platform_windows::instance::{flag, from_flag};
use std::ffi::OsString;
use std::fmt;

/// The command when no argument is given: start, or show the running copy.
const DEFAULT: Command = Command::Show;
/// Every command a flag can name, for the usage line.
const COMMANDS: [Command; 2] = [Command::Show, Command::Quit];
/// The gate-only flag, which only the `gate-input` build takes; the stage settings name it too (the agree test).
pub const SIM_INPUT: &str = "--sim-input";

/// Whose mouse input the hold engine acts on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    /// The hand's, in every shipped build.
    Hand,
    /// The gate tool's tagged input instead; only the `gate-input` build has it.
    #[cfg(feature = "gate-input")]
    Sim,
}

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

/// The command `args` ask for, without the program's own name, and whose input the hold engine takes; nothing means
/// Show with the hand's input.
///
/// # Errors
/// `Unknown` for an argument that is not a flag, `Many` for more than one argument outside the gate-only forms.
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<(Command, Input), ArgError> {
    let args: Vec<OsString> = args.into_iter().collect();
    #[cfg(feature = "gate-input")]
    if let Some(command) = gate(&args) {
        return Ok((command, Input::Sim));
    }
    command(args).map(|c| (c, Input::Hand))
}

/// The command `args` name: none, or one flag.
fn command(args: Vec<OsString>) -> Result<Command, ArgError> {
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

/// Show, when `args` are the gate-only flag alone or followed by `--show`; anything else is the plain parser's.
#[cfg(feature = "gate-input")]
fn gate(args: &[OsString]) -> Option<Command> {
    let show = flag(Command::Show);
    let shows = match args {
        [sim] => sim.to_str() == Some(SIM_INPUT),
        [sim, next] => sim.to_str() == Some(SIM_INPUT) && next.to_str() == Some(show.as_str()),
        _ => false,
    };
    shows.then_some(Command::Show)
}

#[cfg(test)]
mod tests;
