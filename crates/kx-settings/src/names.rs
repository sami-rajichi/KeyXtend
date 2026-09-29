//! Every file name, key name and limit of the settings layer, in one table.

use std::path::{Path, PathBuf};

/// The settings file: plain TOML that holds only the user's changes.
const SETTINGS_FILE: &str = "settings.toml";
/// The last good copy of the settings file.
const PREVIOUS_FILE: &str = "settings.previous.toml";
/// A damaged settings file, kept for the owner to look at.
const BROKEN_FILE: &str = "settings.broken.toml";
/// Where a save writes first, before it takes the place of the settings file.
const TEMP_FILE: &str = "settings.toml.tmp";
/// The folder of the log files.
const LOGS_DIR: &str = "logs";
/// The current log file, inside the logs folder.
const LOG_FILE: &str = "keyxtend.log";
/// The log file of the run before, inside the logs folder.
const LOG_PREVIOUS_FILE: &str = "keyxtend.previous.log";

/// The largest settings file that is read, 1 MiB; the file cannot set its own limit.
pub const MAX_FILE_BYTES: u64 = 1024 * 1024;

/// The key that holds a section's version.
pub const VERSION_KEY: &str = "version";

/// The notice argument naming the module.
pub const ARG_MODULE: &str = "module";
/// The notice argument listing the keys that went back to their defaults.
pub const ARG_KEYS: &str = "keys";
/// The notice argument counting the reset keys that `ARG_KEYS` leaves out; absent when it names all.
pub const ARG_MORE: &str = "more";
/// What joins the keys in `ARG_KEYS`.
pub const KEYS_SEPARATOR: &str = ", ";
/// The most keys one notice names, so a huge file never makes a huge notice.
pub const MAX_NAMED_KEYS: usize = 10;
/// The longest key name a notice shows; a longer one is counted in `ARG_MORE` instead.
pub const MAX_KEY_CHARS: usize = 64;

/// The paths of the files in the data folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Files {
    /// The data folder.
    pub dir: PathBuf,
    /// The settings file.
    pub settings: PathBuf,
    /// The last good copy of the settings file.
    pub previous: PathBuf,
    /// A damaged settings file that was set aside.
    pub broken: PathBuf,
    /// The file a save writes first.
    pub temp: PathBuf,
    /// The logs folder.
    pub logs: PathBuf,
    /// The current log file.
    pub log: PathBuf,
    /// The log file of the run before.
    pub log_previous: PathBuf,
}

impl Files {
    /// Names the files inside `data_dir`, which the platform chose.
    #[must_use]
    pub fn new(data_dir: &Path) -> Self {
        let logs = data_dir.join(LOGS_DIR);
        Self {
            dir: data_dir.to_path_buf(),
            settings: data_dir.join(SETTINGS_FILE),
            previous: data_dir.join(PREVIOUS_FILE),
            broken: data_dir.join(BROKEN_FILE),
            temp: data_dir.join(TEMP_FILE),
            log: logs.join(LOG_FILE),
            log_previous: logs.join(LOG_PREVIOUS_FILE),
            logs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn files() -> Files {
        Files::new(Path::new("data"))
    }

    #[test]
    fn the_settings_files_sit_in_the_data_folder() {
        let f = files();
        assert_eq!(f.settings, Path::new("data").join("settings.toml"));
        assert_eq!(f.previous, Path::new("data").join("settings.previous.toml"));
        assert_eq!(f.broken, Path::new("data").join("settings.broken.toml"));
        assert_eq!(f.temp, Path::new("data").join("settings.toml.tmp"));
    }

    #[test]
    fn the_log_files_sit_in_the_logs_folder() {
        let f = files();
        let logs = Path::new("data").join("logs");
        assert_eq!(f.logs, logs);
        assert_eq!(f.log, logs.join("keyxtend.log"));
        assert_eq!(f.log_previous, logs.join("keyxtend.previous.log"));
    }

    #[test]
    fn every_path_is_different() {
        let f = files();
        let all = [
            &f.dir,
            &f.settings,
            &f.previous,
            &f.broken,
            &f.temp,
            &f.logs,
            &f.log,
            &f.log_previous,
        ];
        let unique: HashSet<_> = all.iter().collect();
        assert_eq!(unique.len(), all.len());
    }

    #[test]
    fn notice_arguments_keep_the_names_translations_use() {
        assert_eq!(
            (ARG_MODULE, ARG_KEYS, ARG_MORE, KEYS_SEPARATOR),
            ("module", "keys", "more", ", ")
        );
    }

    #[test]
    fn the_limit_is_one_mebibyte() {
        assert_eq!(MAX_FILE_BYTES, 1_048_576);
    }
}
