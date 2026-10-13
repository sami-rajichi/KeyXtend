//! Parsing of `cargo xtask sounds render|import`.

use super::{Command, UsageError};

/// Renders every recipe to WAV files.
const RENDER: &str = "render";
/// Converts a recording to the shipping format.
const IMPORT: &str = "import";
/// Flag naming the folder `render` writes to.
const FLAG_OUT: &str = "--out";
/// Flag giving where `import`'s window starts, in ms.
const FLAG_START_MS: &str = "--start-ms";
/// Flag giving `import`'s window length, in ms.
const FLAG_LEN_MS: &str = "--len-ms";
/// Prefix of every flag; a path argument never starts with it.
const FLAG_PREFIX: &str = "--";

/// The usage forms of the `sounds` subcommand, named `sub`.
pub(super) fn forms(sub: &str) -> [String; 2] {
    [
        format!("{sub} {RENDER} [{FLAG_OUT} <dir>]"),
        format!("{sub} {IMPORT} <in.wav> <out.wav> [{FLAG_START_MS} <ms>] [{FLAG_LEN_MS} <ms>]"),
    ]
}

/// Parses the arguments that follow `sounds`.
pub(super) fn parse(args: &[String]) -> Result<Command, UsageError> {
    match args {
        [action] if action == RENDER => Ok(Command::SoundsRender { out: None }),
        [action, flag, dir] if action == RENDER && flag == FLAG_OUT && is_path(dir) => {
            Ok(Command::SoundsRender {
                out: Some(dir.clone()),
            })
        }
        [action, input, output, flags @ ..]
            if action == IMPORT && is_path(input) && is_path(output) =>
        {
            import(input, output, flags)
        }
        _ => Err(UsageError),
    }
}

/// True if `arg` can be a path: it is not empty and is not a flag.
fn is_path(arg: &str) -> bool {
    !arg.is_empty() && !arg.starts_with(FLAG_PREFIX)
}

/// Builds the import command from its paths and its `--start-ms`/`--len-ms` pairs, each given at most once.
fn import(input: &str, output: &str, flags: &[String]) -> Result<Command, UsageError> {
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
        *slot = Some(value.parse::<u32>().map_err(|_| UsageError)?);
    }
    Ok(Command::SoundsImport {
        input: input.to_string(),
        output: output.to_string(),
        start_ms,
        len_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::super::parse as parse_all;
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| (*v).to_string()).collect()
    }

    fn import_of(start_ms: Option<u32>, len_ms: Option<u32>) -> Command {
        Command::SoundsImport {
            input: "in.wav".into(),
            output: "out.wav".into(),
            start_ms,
            len_ms,
        }
    }

    #[test]
    fn render_parses_with_and_without_a_folder() {
        assert_eq!(
            parse_all(&args(&["sounds", "render"])),
            Ok(Command::SoundsRender { out: None })
        );
        assert_eq!(
            parse_all(&args(&["sounds", "render", "--out", "x"])),
            Ok(Command::SoundsRender {
                out: Some("x".into())
            })
        );
    }

    #[test]
    fn render_with_a_bad_flag_or_no_folder_is_a_usage_error() {
        for bad in [
            &["sounds"][..],
            &["sounds", "render", "--out"],
            &["sounds", "render", "--dir", "x"],
            &["sounds", "render", "--out", "--len-ms"],
            &["sounds", "render", "--out", "x", "y"],
        ] {
            assert_eq!(parse_all(&args(bad)), Err(UsageError), "{bad:?}");
        }
    }

    #[test]
    fn import_parses_its_paths_and_flags_in_any_order() {
        let base = ["sounds", "import", "in.wav", "out.wav"];
        assert_eq!(parse_all(&args(&base)), Ok(import_of(None, None)));
        let both = [&base[..], &["--len-ms", "80", "--start-ms", "12"]].concat();
        assert_eq!(parse_all(&args(&both)), Ok(import_of(Some(12), Some(80))));
        let start = [&base[..], &["--start-ms", "0"]].concat();
        assert_eq!(parse_all(&args(&start)), Ok(import_of(Some(0), None)));
    }

    #[test]
    fn import_with_bad_flags_is_a_usage_error() {
        let base = ["sounds", "import", "in.wav", "out.wav"];
        for extra in [
            &["--start-ms"][..],
            &["--start-ms", "-5"],
            &["--start-ms", "ten"],
            &["--start-ms", "1", "--start-ms", "2"],
            &["--fade-ms", "5"],
        ] {
            let all = [&base[..], extra].concat();
            assert_eq!(parse_all(&args(&all)), Err(UsageError), "{extra:?}");
        }
    }

    #[test]
    fn import_needs_two_paths_that_are_not_flags() {
        for bad in [
            &["sounds", "import"][..],
            &["sounds", "import", "in.wav"],
            &["sounds", "import", "--start-ms", "5", "out.wav"],
            &["sounds", "import", "in.wav", "--len-ms", "5"],
        ] {
            assert_eq!(parse_all(&args(bad)), Err(UsageError), "{bad:?}");
        }
    }

    #[test]
    fn usage_names_both_actions_and_every_flag() {
        let message = UsageError.to_string();
        for name in [RENDER, IMPORT, FLAG_OUT, FLAG_START_MS, FLAG_LEN_MS] {
            assert!(message.contains(name), "usage message missing {name}");
        }
    }
}
