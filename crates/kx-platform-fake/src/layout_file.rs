//! The layout table file format: one TOML file per layout, with every code in `0x` hex.

use kx_module_api::ports::{KeyChars, LangFacts, LayoutId, Scan};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// The extension of a layout table file.
pub const FILE_EXT: &str = "toml";
/// The folder of the golden tables, under this crate.
const GOLDEN_DIR: &str = "golden/layouts";
/// What starts every code in a table file.
const HEX_PREFIX: &str = "0x";
/// The base of the codes.
const HEX: u32 = 16;
/// Hex digits of a written scan code.
const SCAN_DIGITS: usize = 4;
/// Hex digits of a written layout id.
const ID_DIGITS: usize = 16;

/// Why a layout table could not be read or written.
#[derive(Debug, Error)]
pub enum LayoutFileError {
    /// The file could not be read.
    #[error("{}: {source}", path.display())]
    Io {
        /// The file.
        path: PathBuf,
        /// What the system said.
        source: std::io::Error,
    },
    /// The text is not a layout table.
    #[error("layout table does not parse: {0}")]
    Toml(#[from] toml::de::Error),
    /// The table could not be written as TOML.
    #[error("layout table does not write: {0}")]
    Write(#[from] toml::ser::Error),
    /// A code is not `0x` and hex digits, or is too big.
    #[error("{0:?} is not a 0x hex code")]
    BadCode(String),
}

/// One layout: its id, its facts, the keys that type a control character, and what each key types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutFile {
    /// The layout.
    pub id: LayoutId,
    /// Its language facts.
    pub facts: LangFacts,
    /// Keys that type a control character, such as Enter, which `KeyChars` shows as empty.
    pub controls: Vec<Scan>,
    /// What each key types.
    pub keys: BTreeMap<Scan, KeyChars>,
}

/// The file as TOML holds it: codes as hex text, plain values before tables.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    id: String,
    #[serde(default)]
    controls: Vec<String>,
    facts: LangFacts,
    #[serde(default)]
    keys: BTreeMap<String, KeyChars>,
}

/// Reads `text` as `0x` and hex digits.
fn hex<T>(
    text: &str,
    parse: fn(&str, u32) -> Result<T, ParseIntError>,
) -> Result<T, LayoutFileError> {
    let bad = || LayoutFileError::BadCode(text.to_owned());
    let digits = text.strip_prefix(HEX_PREFIX).ok_or_else(bad)?;
    if !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(bad());
    }
    parse(digits, HEX).map_err(|_| bad())
}

fn scan(text: &str) -> Result<Scan, LayoutFileError> {
    hex(text, u16::from_str_radix).map(Scan::new)
}

fn scan_text(scan: Scan) -> String {
    format!("{HEX_PREFIX}{:0w$X}", scan.code(), w = SCAN_DIGITS)
}

impl LayoutFile {
    /// Reads a table from TOML text.
    ///
    /// # Errors
    /// `Toml` when the text is not a table, `BadCode` for a code that is not `0x` hex.
    pub fn parse(text: &str) -> Result<Self, LayoutFileError> {
        let raw: Raw = toml::from_str(text)?;
        let controls = raw.controls.iter().map(|c| scan(c));
        let keys = raw.keys.into_iter().map(|(c, k)| Ok((scan(&c)?, k)));
        Ok(Self {
            id: LayoutId::new(hex(&raw.id, u64::from_str_radix)?),
            facts: raw.facts,
            controls: controls.collect::<Result<_, _>>()?,
            keys: keys.collect::<Result<_, LayoutFileError>>()?,
        })
    }

    /// The table as TOML text.
    ///
    /// # Errors
    /// `Write` when TOML cannot hold a value.
    pub fn to_toml(&self) -> Result<String, LayoutFileError> {
        let raw = Raw {
            id: format!("{HEX_PREFIX}{:0w$X}", self.id.get(), w = ID_DIGITS),
            controls: self.controls.iter().map(|&c| scan_text(c)).collect(),
            facts: self.facts.clone(),
            keys: self
                .keys
                .iter()
                .map(|(&c, k)| (scan_text(c), k.clone()))
                .collect(),
        };
        Ok(toml::to_string(&raw)?)
    }

    /// Reads the table file at `path`.
    ///
    /// # Errors
    /// `Io` when the file cannot be read, else as `parse`.
    pub fn read(path: &Path) -> Result<Self, LayoutFileError> {
        let text = std::fs::read_to_string(path).map_err(|source| LayoutFileError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::parse(&text)
    }
}

/// The folder of this crate's golden layout tables.
#[must_use]
pub fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(GOLDEN_DIR)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::ports::Slot;

    fn table() -> LayoutFile {
        let hat = KeyChars {
            base: "^".into(),
            shift: "¨".into(),
            dead: vec![Slot::Base, Slot::Shift],
            ..KeyChars::default()
        };
        let slash = KeyChars {
            base: "/".into(),
            ..KeyChars::default()
        };
        LayoutFile {
            id: LayoutId::new(0xFFFF_FFFF_F002_0409),
            facts: LangFacts {
                iso: "fr".into(),
                native: "français".into(),
                layout: "French".into(),
                rtl: false,
            },
            controls: vec![Scan::new(0x1C)],
            keys: [(Scan::new(0x1A), hat), (Scan::new(0xE035), slash)].into(),
        }
    }

    #[test]
    fn a_table_round_trips_with_hex_codes() {
        let text = table().to_toml().unwrap();
        assert!(text.contains("0xFFFFFFFFF0020409"), "{text}");
        assert!(text.contains("0xE035") && text.contains("0x001C"), "{text}");
        assert_eq!(LayoutFile::parse(&text).unwrap(), table());
    }

    #[test]
    fn a_code_without_hex_form_is_refused() {
        let text = table().to_toml().unwrap();
        for (good, bad) in [("0x001C", "28"), ("0xE035", "0xE0351"), ("0x001A", "0xZZ")] {
            let broken = text.replace(good, bad);
            let err = LayoutFile::parse(&broken).unwrap_err();
            assert!(matches!(err, LayoutFileError::BadCode(_)), "{bad}: {err}");
        }
    }

    #[test]
    fn an_unknown_field_is_refused() {
        let text = format!("extra = 1\n{}", table().to_toml().unwrap());
        let err = LayoutFile::parse(&text).unwrap_err();
        assert!(matches!(err, LayoutFileError::Toml(_)), "{err}");
    }

    #[test]
    fn every_golden_file_parses() {
        let dir = golden_dir();
        let mut seen = 0;
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == FILE_EXT) {
                LayoutFile::read(&path).unwrap();
                seen += 1;
            }
        }
        assert!(seen > 0, "no tables in {}", dir.display());
    }

    #[test]
    fn a_missing_file_names_its_path() {
        let path = golden_dir().join("missing.toml");
        let err = LayoutFile::read(&path).unwrap_err();
        assert!(err.to_string().contains("missing.toml"), "{err}");
    }
}
