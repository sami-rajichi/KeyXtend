//! The keys a layout table holds, from the compiled-in `scans.toml`.

use kx_module_api::ports::Scan;
use serde::Deserialize;
use thiserror::Error;

/// The scan table, compiled in so it is never read from disk.
const TABLE: &str = include_str!("../scans.toml");

/// Why the compiled-in scan table is unusable.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ScanTableError {
    /// The table is not TOML of the expected shape.
    #[error("the scan table does not parse: {0}")]
    Parse(String),
    /// `first` comes after `last`.
    #[error("the scan table's range runs backwards")]
    Reversed,
}

/// The table as TOML holds it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    first: Scan,
    last: Scan,
    extended: Vec<Scan>,
}

/// Every scan code a layout table reads: `first` to `last`, then `extended`.
///
/// # Errors
/// `ScanTableError` when the compiled-in table is broken.
pub fn scans() -> Result<Vec<Scan>, ScanTableError> {
    read(TABLE)
}

/// Reads a scan table from `text`.
fn read(text: &str) -> Result<Vec<Scan>, ScanTableError> {
    let raw: Raw = toml::from_str(text).map_err(|e| ScanTableError::Parse(e.to_string()))?;
    let (first, last) = (raw.first.code(), raw.last.code());
    if first > last {
        return Err(ScanTableError::Reversed);
    }
    Ok((first..=last).map(Scan::new).chain(raw.extended).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kx_platform_fake::{FILE_EXT, LayoutFile, golden_dir};
    use std::collections::BTreeSet;

    #[test]
    fn the_shipped_table_holds_every_captured_key_once() {
        let all = scans().unwrap();
        let once: BTreeSet<Scan> = all.iter().copied().collect();
        assert_eq!(once.len(), all.len(), "a key is listed twice");
        for name in ["en-us", "fr-fr", "ar-tn"] {
            let path = golden_dir().join(name).with_extension(FILE_EXT);
            let file = LayoutFile::read(&path).unwrap();
            assert!(file.keys.keys().all(|k| once.contains(k)), "{name}");
        }
    }

    #[test]
    fn the_range_comes_before_the_extended_keys() {
        let text = "first = 0x02\nlast = 0x04\nextended = [0xE035]";
        let want = [0x02, 0x03, 0x04, 0xE035].map(Scan::new);
        assert_eq!(read(text).unwrap(), want);
    }

    #[test]
    fn a_broken_table_is_an_error_not_a_panic() {
        let reversed = "first = 0x04\nlast = 0x02\nextended = []";
        assert_eq!(read(reversed), Err(ScanTableError::Reversed));
        for bad in [
            "",
            "first = 1\nlast = 2",
            "first = -1\nlast = 2\nextended = []",
        ] {
            assert!(
                matches!(read(bad), Err(ScanTableError::Parse(_))),
                "{bad:?}"
            );
        }
        let extra = "first = 1\nlast = 2\nextended = []\nmore = 1";
        assert!(matches!(read(extra), Err(ScanTableError::Parse(_))));
    }
}
