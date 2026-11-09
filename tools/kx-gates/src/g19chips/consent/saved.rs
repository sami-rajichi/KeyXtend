//! The copy's saved Translate consent, read as the product reads it: the settings file, else its previous copy, else
//! the defaults. A file that is there but does not read, or cannot be checked, stops the gate.
#![cfg(windows)]

use std::io::ErrorKind;
use std::path::Path;

use kx_mod_selection::{ID, SPEC, SelSettings};
use kx_settings::{Files, peek, peek_section};

use crate::product;

/// Why the gate stops on a settings file that is there but does not read.
const UNREAD: &str = "the copy's settings file does not read, so the gate stops before Translate";

/// Whether a file is there, from its metadata read `meta`: "not found" is no file, and any other error stays one.
pub fn there(meta: std::io::Result<()>) -> Result<bool, String> {
    match meta {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!(
            "the copy's settings file cannot be checked ({e}), so the gate stops before Translate"
        )),
    }
}

/// The table the copy reads, each file given as whether it is there and what it read: the `main` file when it is
/// there, else the `previous` copy, else none for the defaults.
pub fn chosen(
    main: (bool, Option<toml::Table>),
    previous: impl FnOnce() -> Result<(bool, Option<toml::Table>), String>,
) -> Result<Option<toml::Table>, String> {
    let (is, read) = if main.0 { main } else { previous()? };
    if is && read.is_none() {
        return Err(UNREAD.into());
    }
    Ok(read)
}

/// The consent in settings table `file`, the default while there is none.
pub fn consent_in(file: Option<&toml::Table>) -> Result<bool, String> {
    let s: SelSettings =
        peek_section(file, ID, &SPEC).map_err(|e| format!("the selection settings: {e}"))?;
    Ok(s.translate_ok)
}

/// The consent of the copy at `program`, from its own data folder.
pub fn consent_of(program: &Path) -> Result<bool, String> {
    let files = product::files(program)?;
    let at = |path: &Path| {
        let is = there(std::fs::metadata(path).map(|_| ()))?;
        let one = Files {
            settings: path.to_path_buf(),
            ..files.clone()
        };
        Ok((is, peek(&one)))
    };
    let table = chosen(at(&files.settings)?, || at(&files.previous))?;
    consent_in(table.as_ref())
}

#[cfg(test)]
mod tests {
    use std::io::Error;

    use super::*;

    /// A table holding `translate_ok`.
    fn given(ok: bool) -> toml::Table {
        format!("[{}]\ntranslate_ok = {ok}\n", ID.as_str())
            .parse()
            .expect("TOML")
    }

    #[test]
    fn only_a_missing_file_counts_as_none_and_a_failed_check_stops_the_gate() {
        assert_eq!(there(Ok(())), Ok(true));
        assert_eq!(there(Err(Error::from(ErrorKind::NotFound))), Ok(false));
        let e = there(Err(Error::from(ErrorKind::PermissionDenied))).expect_err("unchecked");
        assert!(e.contains("stops"), "{e}");
    }

    #[test]
    fn the_settings_file_wins_else_its_previous_copy_as_the_product_reads_them() {
        let unread =
            || -> Result<(bool, Option<toml::Table>), String> { panic!("the copy was read") };
        assert_eq!(
            chosen((true, Some(given(true))), unread),
            Ok(Some(given(true)))
        );
        let e = chosen((true, None), unread).expect_err("main unread");
        assert!(e.contains("stops"), "{e}");
        let kept = chosen((false, None), || Ok((true, Some(given(true)))));
        assert_eq!(kept, Ok(Some(given(true))), "the previous copy");
        assert!(
            chosen((false, None), || Ok((true, None))).is_err(),
            "copy unread"
        );
        assert_eq!(chosen((false, None), || Ok((false, None))), Ok(None));
        let e = chosen((false, None), || Err("unchecked".into()));
        assert_eq!(e, Err("unchecked".into()));
    }

    #[test]
    fn the_consent_is_the_saved_one_else_the_default() {
        assert_eq!(consent_in(Some(&given(true))), Ok(true));
        assert_eq!(consent_in(Some(&given(false))), Ok(false));
        assert_eq!(consent_in(None), Ok(false), "no file yet: the default");
    }
}
