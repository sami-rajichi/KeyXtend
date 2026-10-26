//! Subcommand parsing for the xtask CLI.

mod sounds;
mod words;

use std::fmt;

pub(crate) use self::sounds::SoundsAction;
pub(crate) use self::words::WordsAction;

/// Exit code for a successful run.
pub(crate) const EXIT_SUCCESS: u8 = 0;
/// Exit code when a subcommand finds a problem.
pub(crate) const EXIT_CHECK_FAILED: u8 = 1;
/// Exit code for a usage error: unknown, missing or malformed subcommand.
pub(crate) const EXIT_USAGE_ERROR: u8 = 2;

const SUB_TIDY: &str = "tidy";
const SUB_DCO: &str = "dco";
const SUB_LICENCES: &str = "licences";
const SUB_DIST: &str = "dist";
/// Builds the app in release and stages it with its Qt files.
const SUB_STAGE: &str = "stage";
/// Creates or removes the uiAccess test certificate.
pub(crate) const SUB_DEV_CERT: &str = "dev-cert";
/// Installs or removes a signed test build under Program Files.
const SUB_DEV_INSTALL: &str = "dev-install";
/// Checks the Windows conditions for uiAccess.
const SUB_CHECK_UIACCESS: &str = "check-uiaccess";
/// Renders, imports or joins sounds.
const SUB_SOUNDS: &str = "sounds";
/// Checks the word sources, lists the cleaned words and builds the word packs.
const SUB_WORDS: &str = "words";
/// Flag that turns `dev-cert` and `dev-install` into their undo.
const FLAG_REMOVE: &str = "--remove";
/// Prefix of every flag; a path argument never starts with it.
const FLAG_PREFIX: &str = "--";
/// Flag naming the folder a subcommand writes to, instead of its default.
const FLAG_OUT: &str = "--out";

/// Builds the usage line from the subcommand-name constants, so each name is
/// spelled exactly once.
fn usage_line() -> String {
    let mut forms = vec![
        SUB_TIDY.to_string(),
        format!("{SUB_DCO} <base> <head>"),
        SUB_LICENCES.to_string(),
        SUB_DIST.to_string(),
        SUB_STAGE.to_string(),
        format!("{SUB_DEV_CERT} [{FLAG_REMOVE}]"),
        format!("{SUB_DEV_INSTALL} <folder>"),
        format!("{SUB_DEV_INSTALL} {FLAG_REMOVE} <name>"),
        format!("{SUB_CHECK_UIACCESS} <exe>"),
    ];
    forms.extend(sounds::forms(SUB_SOUNDS));
    forms.extend(words::forms(SUB_WORDS));
    format!("usage: cargo xtask <{}>", forms.join("|"))
}

/// The subcommand that checks the word sources and downloads the missing ones, from the names above.
pub(crate) fn words_fetch() -> String {
    format!("{SUB_WORDS} {}", words::FETCH)
}

/// A parsed xtask subcommand.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Command {
    /// Run the workspace tidy checks.
    Tidy,
    /// Check DCO sign-offs between two git revisions.
    Dco {
        /// The base revision, exclusive.
        base: String,
        /// The head revision, inclusive.
        head: String,
    },
    /// Check third-party licences.
    Licences,
    /// Build release distribution artifacts.
    Dist,
    /// Build the app in release and stage it with its Qt files.
    Stage,
    /// Create the test certificate, or remove it.
    DevCert {
        /// Remove instead of create.
        remove: bool,
    },
    /// Sign a build folder and install it under Program Files.
    DevInstall {
        /// The build folder.
        folder: String,
    },
    /// Delete an installed test build.
    DevUninstall {
        /// The install name.
        name: String,
    },
    /// Check the three Windows conditions for uiAccess.
    CheckUiAccess {
        /// The installed program.
        exe: String,
    },
    /// Render, import or join sounds.
    Sounds(SoundsAction),
    /// Check the word sources, list one language's words, or build the packs.
    Words(WordsAction),
}

/// A command-line usage error. Its `Display` is the usage line.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct UsageError;

impl fmt::Display for UsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", usage_line())
    }
}

/// True if `arg` can be a path: it is not empty and is not a flag.
fn is_path(arg: &str) -> bool {
    !arg.is_empty() && !arg.starts_with(FLAG_PREFIX)
}

/// Parses xtask's arguments (excluding the program name) into a [`Command`].
pub(crate) fn parse(args: &[String]) -> Result<Command, UsageError> {
    match args {
        [name] if name == SUB_TIDY => Ok(Command::Tidy),
        [name] if name == SUB_LICENCES => Ok(Command::Licences),
        [name] if name == SUB_DIST => Ok(Command::Dist),
        [name] if name == SUB_STAGE => Ok(Command::Stage),
        [name, base, head] if name == SUB_DCO => Ok(Command::Dco {
            base: base.clone(),
            head: head.clone(),
        }),
        [name] if name == SUB_DEV_CERT => Ok(Command::DevCert { remove: false }),
        [name, flag] if name == SUB_DEV_CERT && flag == FLAG_REMOVE => {
            Ok(Command::DevCert { remove: true })
        }
        [name, flag, target] if name == SUB_DEV_INSTALL && flag == FLAG_REMOVE => {
            Ok(Command::DevUninstall {
                name: target.clone(),
            })
        }
        [name, folder] if name == SUB_DEV_INSTALL && folder != FLAG_REMOVE => {
            Ok(Command::DevInstall {
                folder: folder.clone(),
            })
        }
        [name, exe] if name == SUB_CHECK_UIACCESS => {
            Ok(Command::CheckUiAccess { exe: exe.clone() })
        }
        [name, rest @ ..] if name == SUB_SOUNDS => sounds::parse(rest).map(Command::Sounds),
        [name, rest @ ..] if name == SUB_WORDS => words::parse(rest).map(Command::Words),
        _ => Err(UsageError),
    }
}

#[cfg(test)]
mod tests;
