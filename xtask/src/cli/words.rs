//! Parsing of `cargo xtask words fetch|list|build|index`.

use super::{UsageError, is_path};

/// Checks every source file, downloading the missing ones.
const FETCH: &str = "fetch";
/// Prints one language's word counts and coverage line.
const LIST: &str = "list";
/// Builds every pack and its credits.
const BUILD: &str = "build";
/// Flag naming the source folder; it is always given, never assumed.
const FLAG_SRC: &str = "--src";
/// Flag naming the folder the packs go to, instead of the manifest's.
const FLAG_OUT: &str = "--out";
/// Rewrites the pack index from the packs in a folder.
const INDEX: &str = "index";
/// Flag naming the packs folder, instead of the manifest's.
const FLAG_PACKS: &str = "--packs";

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
    /// Build every pack from the sources under `src` into `out`, or the manifest's folder.
    Build {
        /// The source folder.
        src: String,
        /// The folder the packs go to.
        out: Option<String>,
    },
    /// Rewrite the pack index from the packs in `packs`, or the manifest's folder.
    Index {
        /// The packs folder.
        packs: Option<String>,
    },
}

/// The usage forms of the `words` subcommand, named `sub`.
pub(super) fn forms(sub: &str) -> [String; 4] {
    [
        format!("{sub} {FETCH} {FLAG_SRC} <dir>"),
        format!("{sub} {LIST} <iso> {FLAG_SRC} <dir>"),
        format!("{sub} {BUILD} {FLAG_SRC} <dir> [{FLAG_OUT} <dir>]"),
        format!("{sub} {INDEX} [{FLAG_PACKS} <dir>]"),
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
        [action, flag, src, rest @ ..] if action == BUILD && flag == FLAG_SRC && is_path(src) => {
            let out = match rest {
                [] => None,
                [flag, out] if flag == FLAG_OUT && is_path(out) => Some(out.clone()),
                _ => return Err(UsageError),
            };
            Ok(WordsAction::Build {
                src: src.clone(),
                out,
            })
        }
        [action] if action == INDEX => Ok(WordsAction::Index { packs: None }),
        [action, flag, packs] if action == INDEX && flag == FLAG_PACKS && is_path(packs) => {
            Ok(WordsAction::Index {
                packs: Some(packs.clone()),
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
    fn build_parses_with_its_source_and_an_optional_out_folder() {
        assert_eq!(
            parse_all(&args(&["words", "build", "--src", "w"])),
            Ok(Command::Words(WordsAction::Build {
                src: "w".into(),
                out: None
            }))
        );
        assert_eq!(
            parse_all(&args(&["words", "build", "--src", "w", "--out", "o"])),
            Ok(Command::Words(WordsAction::Build {
                src: "w".into(),
                out: Some("o".into())
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
            &["words", "build"],
            &["words", "build", "--out", "o"],
            &["words", "build", "--src", "w", "--out"],
            &["words", "build", "--src", "w", "--out", "--src"],
            &["words", "build", "--src", "w", "--src", "o"],
            &["words", "build", "--src", "w", "o"],
            &["words", "index", "--packs"],
            &["words", "index", "--src", "w"],
            &["words", "index", "--packs", "--packs"],
            &["words", "index", "--packs", "p", "x"],
        ] {
            assert_eq!(parse_all(&args(bad)), Err(UsageError), "{bad:?}");
        }
    }

    #[test]
    fn index_parses_with_an_optional_packs_folder() {
        assert_eq!(
            parse_all(&args(&["words", "index"])),
            Ok(Command::Words(WordsAction::Index { packs: None }))
        );
        assert_eq!(
            parse_all(&args(&["words", "index", "--packs", "p"])),
            Ok(Command::Words(WordsAction::Index {
                packs: Some("p".into())
            }))
        );
    }
}
