//! The command line: nothing or `--show` shows the keyboard, `--quit` ends a running copy, and only the gate-only builds
//! take more: `gate-input` takes `--sim-input`, alone or before `--show`, and `gate-voice` takes `--sound` and one
//! absolute WAV path. Anything else is refused before Qt starts, so no Qt option (such as a plugin path) reaches the
//! uiAccess process.

use kx_platform::ui::Command;
use kx_platform_windows::instance::{flag, from_flag};
use std::ffi::OsString;
use std::fmt;

#[cfg(feature = "gate-voice")]
pub use kx_ipc::SOUND_FLAG;

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
    #[cfg(feature = "gate-voice")]
    if played(&args).is_some() {
        return Ok((Command::Show, Input::Hand));
    }
    command(args).map(|c| (c, Input::Hand))
}

/// What the command line asks beside its command: whose input the hold engine takes and, in the voice gate build, the
/// clip its worker plays.
pub struct Flags {
    /// Whose input the hold engine takes.
    pub input: Input,
    /// The clip played in place of the microphone.
    #[cfg(feature = "gate-voice")]
    pub clip: Option<std::path::PathBuf>,
}

/// The command `raw` names and the flags beside it.
///
/// # Errors
/// As [`parse`].
pub fn read(raw: Vec<OsString>) -> Result<(Command, Flags), ArgError> {
    #[cfg(feature = "gate-voice")]
    let clip = clip(&raw);
    let (command, input) = parse(raw)?;
    let flags = Flags {
        input,
        #[cfg(feature = "gate-voice")]
        clip,
    };
    Ok((command, flags))
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

/// The clip the voice gate build's worker plays, when `args` are its flag and one absolute path.
#[cfg(feature = "gate-voice")]
#[must_use]
pub fn clip(args: &[OsString]) -> Option<std::path::PathBuf> {
    played(args).map(std::path::PathBuf::from)
}

/// The path after the sound flag, when `args` are exactly the flag and one absolute UTF-8 path, as the worker takes.
#[cfg(feature = "gate-voice")]
fn played(args: &[OsString]) -> Option<&OsString> {
    match args {
        [sound, file] if sound.to_str() == Some(SOUND_FLAG) => {
            let whole = file.to_str().is_some() && std::path::Path::new(file).is_absolute();
            whole.then_some(file)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests;
