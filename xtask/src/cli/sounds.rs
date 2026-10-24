//! Parsing of `cargo xtask sounds render|import|join`.

use super::{UsageError, is_path};

/// Renders every recipe to WAV files.
const RENDER: &str = "render";
/// Converts a recording to the shipping format.
const IMPORT: &str = "import";
/// Mixes two shipping-format files into one.
const JOIN: &str = "join";
/// Flag naming the folder `render` writes to.
const FLAG_OUT: &str = "--out";
/// Flag giving where `import`'s window starts, in ms.
const FLAG_START_MS: &str = "--start-ms";
/// Flag giving `import`'s window length, in ms.
const FLAG_LEN_MS: &str = "--len-ms";
/// Flag giving where `join`'s second file starts, in ms after the first.
const FLAG_AT_MS: &str = "--at-ms";

/// What `cargo xtask sounds` does.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SoundsAction {
    /// Render every sound recipe to WAV files.
    Render {
        /// The output folder; without it, the one `[workspace.metadata.sounds]` names.
        out: Option<String>,
    },
    /// Convert a recording to the shipping sound format.
    Import {
        /// The recording.
        input: String,
        /// The converted file.
        output: String,
        /// Where the kept window starts, in ms.
        start_ms: Option<u32>,
        /// The kept window's length, in ms.
        len_ms: Option<u32>,
    },
    /// Mix two shipping-format files, the second starting `at_ms` after the first.
    Join {
        /// The file that starts at zero.
        first: String,
        /// The file that starts at `at_ms`.
        second: String,
        /// The mixed file.
        output: String,
        /// Where the second file starts, in ms.
        at_ms: u32,
    },
}

/// The usage forms of the `sounds` subcommand, named `sub`.
pub(super) fn forms(sub: &str) -> [String; 3] {
    [
        format!("{sub} {RENDER} [{FLAG_OUT} <dir>]"),
        format!("{sub} {IMPORT} <in.wav> <out.wav> [{FLAG_START_MS} <ms>] [{FLAG_LEN_MS} <ms>]"),
        format!("{sub} {JOIN} <first.wav> <second.wav> <out.wav> {FLAG_AT_MS} <ms>"),
    ]
}

/// Parses the arguments that follow `sounds`.
pub(super) fn parse(args: &[String]) -> Result<SoundsAction, UsageError> {
    match args {
        [action] if action == RENDER => Ok(SoundsAction::Render { out: None }),
        [action, flag, dir] if action == RENDER && flag == FLAG_OUT && is_path(dir) => {
            Ok(SoundsAction::Render {
                out: Some(dir.clone()),
            })
        }
        [action, input, output, flags @ ..]
            if action == IMPORT && is_path(input) && is_path(output) =>
        {
            import(input, output, flags)
        }
        [action, first, second, output, flag, at]
            if action == JOIN
                && flag == FLAG_AT_MS
                && [first, second, output].iter().all(|path| is_path(path)) =>
        {
            Ok(SoundsAction::Join {
                first: first.clone(),
                second: second.clone(),
                output: output.clone(),
                at_ms: ms(at)?,
            })
        }
        _ => Err(UsageError),
    }
}

/// A whole number of milliseconds, or a usage error.
fn ms(value: &str) -> Result<u32, UsageError> {
    value.parse().map_err(|_| UsageError)
}

/// Builds the import action from its paths and its `--start-ms`/`--len-ms` pairs, each given at most once.
fn import(input: &str, output: &str, flags: &[String]) -> Result<SoundsAction, UsageError> {
    let (mut start_ms, mut len_ms) = (None, None);
    for pair in flags.chunks(2) {
        let [flag, value] = pair else {
            return Err(UsageError);
        };
        let slot = match flag.as_str() {
            FLAG_START_MS => &mut start_ms,
            FLAG_LEN_MS => &mut len_ms,
            _ => return Err(UsageError),
        };
        if slot.is_some() {
            return Err(UsageError);
        }
        *slot = Some(ms(value)?);
    }
    Ok(SoundsAction::Import {
        input: input.to_string(),
        output: output.to_string(),
        start_ms,
        len_ms,
    })
}

#[cfg(test)]
mod tests;
