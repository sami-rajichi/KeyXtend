//! Reads kx-gates' own settings files from one folder as one table, where each section sits in one file only.
#![cfg(windows)]

use std::collections::BTreeMap;
use std::path::Path;

use crate::cfgfile::OWN;

/// The settings file at `path`, parsed.
fn read_table(path: &Path) -> Result<toml::Table, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    text.parse().map_err(|e| format!("{}: {e}", path.display()))
}

/// The sections of each named file's table, as one table; a section in two files is refused, naming both.
fn merge<'a>(files: Vec<(&'a str, toml::Table)>) -> Result<toml::Table, String> {
    let mut all = toml::Table::new();
    let mut found_in: BTreeMap<String, &'a str> = BTreeMap::new();
    for (file, table) in files {
        for (section, value) in table {
            if let Some(first) = found_in.insert(section.clone(), file) {
                return Err(format!("[{section}] is in both {first} and {file}"));
            }
            all.insert(section, value);
        }
    }
    Ok(all)
}

/// kx-gates' own settings files in `dir`, read in order as one table.
pub fn read(dir: &Path) -> Result<toml::Table, String> {
    let read_one = |name| read_table(&dir.join(name)).map(|table| (name, table));
    merge(OWN.into_iter().map(read_one).collect::<Result<_, _>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfgfile::{FILE, P3_FILE, P4_FILE};

    fn table(text: &str) -> toml::Table {
        text.parse().expect("parses")
    }

    #[test]
    fn each_later_file_adds_sections_and_never_replaces_one() {
        let files = |p4: &str| {
            vec![
                (FILE, table("[g2]\nclicks = 1\n")),
                (P3_FILE, table("[g8]\nclicks = 2\n")),
                (P4_FILE, table(p4)),
            ]
        };
        let all = merge(files("[gate_input]\nstart_ms = 3\n")).expect("no overlap");
        assert_eq!(all.keys().collect::<Vec<_>>(), ["g2", "g8", "gate_input"]);
        let e = merge(files("[g8]\nclicks = 4\n")).expect_err("g8 twice");
        assert!(
            e.contains("[g8]") && e.contains(P3_FILE) && e.contains(P4_FILE),
            "{e}"
        );
        assert!(!e.contains(FILE), "names only the two files: {e}");
    }

    #[test]
    fn the_shipped_files_read_as_one_table() {
        let all = read(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("reads");
        for section in ["paths", "mouse_guard", "gate_input"] {
            assert!(all.contains_key(section), "{section}");
        }
    }

    #[test]
    fn a_missing_file_is_an_error_naming_it() {
        let e = read(Path::new("no/such/folder")).expect_err("nothing there");
        assert!(e.contains(FILE), "{e}");
    }
}
