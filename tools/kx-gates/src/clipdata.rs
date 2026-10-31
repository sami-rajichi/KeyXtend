//! The test build's portable data folder around a clipboard gate (Rulings 57 and 60): a folder the gate makes carries
//! its marker and goes at the end, a marked leftover of a hard stop is made afresh, and an unmarked one is never removed.
#![cfg(windows)]

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use kx_platform::PORTABLE_DIR;
use kx_settings::Files;

use crate::product;

/// Deletes folder `dir`, when there is one.
pub(crate) fn remove(dir: &Path) -> Result<(), String> {
    match fs::remove_dir_all(dir) {
        Err(e) if e.kind() != ErrorKind::NotFound => Err(format!("{}: {e}", dir.display())),
        _ => Ok(()),
    }
}

/// True when `name` is, in any case, the `history` folder or a name the product's settings layer writes in its data
/// folder, so a marker by that name would mark a folder the gate never made.
pub(crate) fn taken(name: &str, history: &str) -> bool {
    let f = Files::new(Path::new(""));
    let product = [&f.settings, &f.previous, &f.broken, &f.temp, &f.logs];
    let mut names = product.iter().filter_map(|p| p.to_str()).chain([history]);
    names.any(|n| n.eq_ignore_ascii_case(name))
}

/// True when data folder `data` holds the gate's `marker`, so the gate made it.
pub(crate) fn marked(data: &Path, marker: &str) -> bool {
    data.join(marker).is_file()
}

/// Readies data folder `data`: a marked leftover is deleted, a missing folder is made with `marker` in it, and an
/// unmarked one stays as it is. True when this call made it.
pub(crate) fn prepare(data: &Path, marker: &str) -> Result<bool, String> {
    if marked(data, marker) {
        remove(data)?;
    }
    if data.exists() {
        return Ok(false);
    }
    let fail = |e: std::io::Error| format!("{}: {e}", data.display());
    fs::create_dir_all(data).map_err(fail)?;
    fs::write(data.join(marker), b"").map_err(fail)?;
    Ok(true)
}

/// Error `e`, after deleting data folder `data` when it carries `marker`.
pub(crate) fn undone(data: &Path, marker: &str, e: String) -> String {
    if !marked(data, marker) {
        return e;
    }
    match remove(data) {
        Err(left) => format!("{e}; {left}"),
        Ok(()) => e,
    }
}

/// Ends a run on data folder `data`: a marked folder is deleted, an unmarked one only loses its `history` folder.
pub(crate) fn finish(data: &Path, marker: &str, history: &str) -> Result<(), String> {
    if marked(data, marker) {
        remove(data)
    } else {
        remove(&data.join(history))
    }
}

/// The data folder beside `program`, readied with `marker`, so the copy is portable, once the product reads it there.
pub(crate) fn own(program: &Path, marker: &str) -> Result<PathBuf, String> {
    let data = program
        .parent()
        .ok_or("the test build has no folder")?
        .join(PORTABLE_DIR);
    prepare(&data, marker)?;
    match product::files(program) {
        Ok(f) if f.dir == data => Ok(data),
        Ok(_) => {
            let e = format!(
                "{} is not the copy's data folder, so it would use another",
                data.display()
            );
            Err(undone(&data, marker, e))
        }
        Err(e) => Err(undone(&data, marker, e)),
    }
}

#[cfg(test)]
mod tests {
    use kx_test_support::tempdir::TempDir;

    use super::*;

    const MARK: &str = "made-by-test";
    const HISTORY: &str = "clipboard";

    /// A temp folder and its `data` folder path, not made yet.
    fn data() -> (TempDir, PathBuf) {
        let dir = TempDir::new("gates-clipdata").expect("a temp folder");
        let data = dir.path().join(PORTABLE_DIR);
        (dir, data)
    }

    /// Fills `data` with a settings file and a history file, as a run leaves them.
    fn fill(data: &Path) {
        fs::create_dir_all(data.join(HISTORY)).expect("history folder");
        fs::write(data.join("settings.toml"), "x = 1").expect("settings");
        fs::write(data.join(HISTORY).join("list"), "entries").expect("history");
    }

    #[test]
    fn a_missing_folder_is_made_marked_and_goes_at_the_end() {
        let (_dir, data) = data();
        assert_eq!(prepare(&data, MARK), Ok(true));
        assert!(marked(&data, MARK));
        fill(&data);
        assert_eq!(finish(&data, MARK, HISTORY), Ok(()));
        assert!(!data.exists());
    }

    #[test]
    fn a_marked_leftover_is_made_afresh_and_still_marked() {
        let (_dir, data) = data();
        prepare(&data, MARK).expect("made");
        fill(&data);
        assert_eq!(prepare(&data, MARK), Ok(true));
        assert!(marked(&data, MARK));
        assert!(!data.join("settings.toml").exists() && !data.join(HISTORY).exists());
    }

    #[test]
    fn an_unmarked_folder_stays_and_only_loses_its_history() {
        let (_dir, data) = data();
        fill(&data);
        assert_eq!(prepare(&data, MARK), Ok(false));
        assert!(!marked(&data, MARK));
        assert_eq!(undone(&data, MARK, "boom".into()), "boom");
        assert!(
            data.join("settings.toml").exists(),
            "undone never removes it"
        );
        assert_eq!(finish(&data, MARK, HISTORY), Ok(()));
        assert!(data.join("settings.toml").exists() && !data.join(HISTORY).exists());
    }

    #[test]
    fn the_products_own_names_and_the_history_are_taken_in_any_case() {
        for name in [
            "settings.toml",
            "SETTINGS.previous.toml",
            "settings.broken.toml",
            "settings.toml.tmp",
            "Logs",
        ] {
            assert!(taken(name, HISTORY), "{name}");
        }
        assert!(taken("Clipboard", HISTORY));
        assert!(!taken(MARK, HISTORY));
        assert!(!taken("settings", HISTORY), "only whole names");
    }

    #[test]
    fn undone_deletes_only_a_marked_folder() {
        let (_dir, data) = data();
        prepare(&data, MARK).expect("made");
        fill(&data);
        assert_eq!(undone(&data, MARK, "boom".into()), "boom");
        assert!(!data.exists());
        assert_eq!(remove(&data), Ok(()), "a missing folder is no error");
    }
}
