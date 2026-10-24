//! Parsing of `cargo xtask words fetch|list`.

use super::{UsageError, is_path};

/// Checks every source file, downloading the missing ones.
const FETCH: &str = "fetch";
/// Prints one language's word counts and coverage line.
const LIST: &str = "list";
/// Flag naming the source folder; it is always given, never assumed.
const FLAG_SRC: &str = "--src";

/// What `cargo xtask words` does.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum WordsAction {
    /// Check every source under `src`.
    Fetch {
        /// The source folder.
        src: String,
    },
    /// Print the word list counts of `iso`.
    List {
        /// The language code.
        iso: String,
        /// The source folder.
        src: String,
    },
}

/// The usage forms of the `words` subcommand, named `sub`.
pub(super) fn forms(sub: &str) -> [String; 2] {
    [
        format!("{sub} {FETCH} {FLAG_SRC} <dir>"),
        format!("{sub} {LIST} <iso> {FLAG_SRC} <dir>"),
    ]
}

/// Parses the arguments that follow `words`.
pub(super) fn parse(args: &[String]) -> Result<WordsAction, UsageError> {
    match args {
        [action, flag, src] if action == FETCH && flag == FLAG_SRC && is_path(src) => {
            Ok(WordsAction::Fetch { src: src.clone() })
        }
        [action, iso, flag, src]
            if action == LIST && flag == FLAG_SRC && is_path(iso) && is_path(src) =>
        {
            Ok(WordsAction::List {
                iso: iso.clone(),
                src: src.clone(),
            })
        }
        _ => Err(UsageError),
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Command, parse as parse_all};
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| (*v).to_string()).collect()
    }

    #[test]
    fn fetch_and_list_parse_with_their_source_folder() {
        assert_eq!(
            parse_all(&args(&["words", "fetch", "--src", "D:/w"])),
            Ok(Command::Words(WordsAction::Fetch { src: "D:/w".into() }))
        );
        assert_eq!(
            parse_all(&args(&["words", "list", "ar", "--src", "w"])),
            Ok(Command::Words(WordsAction::List {
                iso: "ar".into(),
                src: "w".into()
            }))
        );
    }

    #[test]
    fn a_missing_source_folder_or_a_bad_flag_is_a_usage_error() {
        for bad in [
            &["words"][..],
            &["words", "fetch"],
            &["words", "fetch", "--src"],
            &["words", "fetch", "--out", "w"],
            &["words", "fetch", "--src", "--src"],
            &["words", "list", "en"],
            &["words", "list", "--src", "w"],
            &["words", "list", "en", "--src", "w", "x"],
            &["words", "build", "--src", "w"],
        ] {
            assert_eq!(parse_all(&args(bad)), Err(UsageError), "{bad:?}");
        }
    }
}
