//! Tools for uiAccess test builds: the stage, a test certificate, a Program Files install with its shortcut, and a check.

pub(crate) mod admin;
pub(crate) mod cert;
pub(crate) mod check;
pub(crate) mod config;
pub(crate) mod install;
pub(crate) mod launcher;
pub(crate) mod paths;
pub(crate) mod ps;
pub(crate) mod script;
pub(crate) mod sdk;
pub(crate) mod shortcut;
pub(crate) mod stage;

use std::fmt;

use self::stage::StageError;
use crate::cli::SUB_DEV_CERT as DEV_CERT;
use crate::workspace::LoadError;

/// Why a dev-tools subcommand failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DevError {
    /// The dev-tools settings could not be loaded.
    Config(String),
    /// No SDK version folder contains the tool.
    NoTool {
        /// The tool's file name.
        tool: String,
        /// The SDK folder that was searched.
        root: String,
    },
    /// A program could not be started.
    Spawn {
        /// The program's name.
        program: String,
        /// Why it could not start.
        reason: String,
    },
    /// A step ran and failed.
    Failed {
        /// The step's name.
        step: String,
        /// The first line of its error output.
        detail: String,
    },
    /// The owner cancelled the Windows prompt.
    Cancelled,
    /// Script output was not a certificate thumbprint.
    BadThumbprint(String),
    /// A file could not be read or written.
    Io(String),
    /// An install name is not one plain folder name.
    BadName(String),
    /// A required environment variable is not set.
    MissingEnv(String),
    /// No valid test certificate exists yet.
    NoCert,
    /// The given path is not a folder.
    NotAFolder(String),
    /// The given path is not a file.
    NotAFile(String),
    /// The build folder and the install target overlap.
    Overlap(String),
    /// The build folder is larger than the configured limit.
    TooBig {
        /// Its size in MB.
        mb: u64,
        /// The limit in MB.
        max: u64,
    },
    /// The build folder holds a link or junction, which the copy would follow.
    Link(String),
    /// The stage, or the build folder an install copies, was refused.
    Stage(StageError),
    /// The stage failed and was removed, so it never looks installable.
    StageRemoved(Box<DevError>),
    /// The shortcut would land outside the user's profile.
    NotInProfile(String),
}

impl From<LoadError> for DevError {
    fn from(err: LoadError) -> Self {
        Self::Config(err.to_string())
    }
}

impl From<std::io::Error> for DevError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl fmt::Display for DevError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(reason) => write!(f, "dev-tools settings: {reason}"),
            Self::NoTool { tool, root } => {
                write!(f, "{tool} not found under {root}; install the Windows SDK")
            }
            Self::Spawn { program, reason } => write!(f, "could not run {program}: {reason}"),
            Self::Failed { step, detail } => write!(f, "{step} failed: {detail}"),
            Self::Cancelled => write!(f, "cancelled at the Windows prompt; nothing was changed"),
            Self::BadThumbprint(text) => write!(f, "not a certificate thumbprint: {text:?}"),
            Self::Io(reason) => write!(f, "file error: {reason}"),
            Self::BadName(name) => write!(f, "{name:?} is not a plain folder name"),
            Self::MissingEnv(name) => write!(f, "environment variable {name} is not set"),
            Self::NoCert => write!(f, "no test certificate; run `cargo xtask {DEV_CERT}` first"),
            Self::NotAFolder(path) => write!(f, "{path} is not a folder"),
            Self::NotAFile(path) => write!(f, "{path} is not a file"),
            Self::Overlap(path) => write!(f, "{path} overlaps the install target; build elsewhere"),
            Self::TooBig { mb, max } => write!(f, "{mb} MB is over the {max} MB limit"),
            Self::Link(path) => write!(f, "{path} is a link; remove it from the build folder"),
            Self::Stage(why) => write!(f, "{why}"),
            Self::StageRemoved(why) => write!(f, "{why}; the stage was removed"),
            Self::NotInProfile(path) => write!(f, "{path} is outside the user's profile"),
        }
    }
}
