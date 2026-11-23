//! Reads kx-gates' own settings files from one folder as one table, where each section sits in one file only, then
//! this PC's local file there, which may add only its own keys.
#![cfg(windows)]

use std::collections::BTreeMap;
use std::path::Path;

use crate::cfgfile::{LOCAL_FILE, LOCAL_KEYS, OWN};

/// The settings file at `path`, parsed; a broken local file is named with its line and column alone, as its text
/// may hold the owner's folder.
fn read_table(path: &Path) -> Result<toml::Table, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let local = path.file_name().is_some_and(|n| n == LOCAL_FILE);
    text.parse().map_err(|e: toml::de::Error| match e.span() {
        Some(span) if local => format!("{LOCAL_FILE}: no TOML at {}", place(&text, span.start)),
        None if local => format!("{LOCAL_FILE}: no TOML"),
        _ => format!("{}: {e}", path.display()),
    })
}

/// The line and column of byte `at` in `text`, counted from 1.
fn place(text: &str, at: usize) -> String {
    let before = text.get(..at).unwrap_or(text);
    let line = before.matches('\n').count() + 1;
    let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
    format!("line {line}, column {column}")
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

/// kx-gates' own settings files in `dir`, read in order as one table, with the local file in `dir`, if any.
pub fn read(dir: &Path) -> Result<toml::Table, String> {
    let read_one = |name| read_table(&dir.join(name)).map(|table| (name, table));
    let mut all = merge(OWN.into_iter().map(read_one).collect::<Result<_, _>>()?)?;
    let local = dir.join(LOCAL_FILE);
    if local.is_file() {
        add_local(&mut all, read_table(&local)?)?;
    }
    Ok(all)
}

/// Adds `local`'s keys to their sections in `all`: only keys `LOCAL_KEYS` names, never one a tracked file sets.
fn add_local(all: &mut toml::Table, local: toml::Table) -> Result<(), String> {
    for (section, value) in local {
        let toml::Value::Table(keys) = value else {
            return Err(format!("{LOCAL_FILE}: {section} must be a section"));
        };
        for (key, value) in keys {
            if !LOCAL_KEYS.contains(&(section.as_str(), key.as_str())) {
                return Err(format!(
                    "{LOCAL_FILE} may set only {LOCAL_KEYS:?}, not {section}.{key}"
                ));
            }
            let into = all.get_mut(&section).and_then(toml::Value::as_table_mut);
            let into = into.ok_or_else(|| format!("{LOCAL_FILE}: no tracked [{section}]"))?;
            if into.insert(key.clone(), value).is_some() {
                return Err(format!(
                    "{section}.{key} is set in a tracked file and in {LOCAL_FILE}"
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use kx_test_support::tempdir::TempDir;

    use super::*;
    use crate::cfgfile::{FILE, LOCAL_FILE, P3_FILE, P4_FILE};

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

    /// The bench's section without the local key, as the tracked files hold it.
    fn tracked() -> toml::Table {
        table("[g22_bench]\nworker = \"w.exe\"\n")
    }

    #[test]
    fn the_local_file_adds_only_its_own_keys_to_their_sections() {
        let mut all = tracked();
        add_local(&mut all, table("[g22_bench]\nmodels_dir = 'E:\\models'\n")).expect("its key");
        assert_eq!(all["g22_bench"]["models_dir"].as_str(), Some(r"E:\models"));
        assert_eq!(all["g22_bench"]["worker"].as_str(), Some("w.exe"));
    }

    #[test]
    fn the_local_file_sets_the_daily_folder_of_g22s_cloud_line() {
        let mut all = table("[g22_cloud]\nopen_ms = 1\n");
        let local = table("[g22_cloud]\ndaily_dir = 'E:\\daily'\n");
        add_local(&mut all, local).expect("its key");
        assert_eq!(all["g22_cloud"]["daily_dir"].as_str(), Some(r"E:\daily"));
    }

    #[test]
    fn the_local_file_never_replaces_a_tracked_key_or_sets_another() {
        let refused = [
            "[g22_bench]\nworker = \"x.exe\"\n",
            "[g8]\nprogram = \"x.exe\"\n",
            "[g22_bench]\nmodels_dir = 'E:\\a'\nworker = \"x.exe\"\n",
            "models_dir = 'E:\\a'\n",
        ];
        for local in refused {
            let e = add_local(&mut tracked(), table(local)).expect_err(local);
            assert!(e.contains(LOCAL_FILE), "{e}");
        }
        let mut set = table("[g22_bench]\nmodels_dir = 'E:\\a'\n");
        let twice = add_local(&mut set, table("[g22_bench]\nmodels_dir = 'E:\\b'\n"));
        assert!(twice.is_err(), "a key a tracked file sets");
    }

    /// A folder holding a copy of the tracked settings files.
    fn copied() -> TempDir {
        let dir = TempDir::new("kx-gates-local").expect("a folder");
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        for name in OWN {
            std::fs::copy(crate_dir.join(name), dir.path().join(name)).expect("a copy");
        }
        dir
    }

    #[test]
    fn a_broken_local_file_names_its_line_and_column_never_its_text() {
        let (dir, folder) = (copied(), r"E:\Users\someone\daily");
        let broken = format!("[g22_cloud]\ndaily_dir = '{folder}\n");
        std::fs::write(dir.path().join(LOCAL_FILE), broken).expect("written");
        let e = read(dir.path()).expect_err("broken");
        assert!(e.contains(LOCAL_FILE) && e.contains("line 2"), "{e}");
        assert!(!e.contains("someone") && !e.contains("daily_dir"), "{e}");
    }

    #[test]
    fn the_local_file_is_read_only_from_the_settings_folder() {
        let dir = copied();
        let before = read(dir.path()).expect("reads");
        assert!(
            before["g22_bench"].get("models_dir").is_none(),
            "none beside it yet"
        );
        let local = "[g22_bench]\nmodels_dir = 'E:\\models'\n";
        std::fs::write(dir.path().join(LOCAL_FILE), local).expect("written");
        let after = read(dir.path()).expect("reads");
        assert_eq!(
            after["g22_bench"]["models_dir"].as_str(),
            Some(r"E:\models")
        );
    }
}
