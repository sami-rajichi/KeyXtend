//! The layout table file format: one TOML file per layout, with every code in `0x` hex text.

use kx_module_api::ports::{CodeError, KeyChars, LangFacts, LayoutId, Scan};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// The extension of a layout table file.
pub const FILE_EXT: &str = "toml";
/// The folder of the golden tables, under this crate.
const GOLDEN_DIR: &str = "golden/layouts";

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
    /// A scan code is not `0x` and hex digits, or is too big.
    #[error(transparent)]
    BadCode(#[from] CodeError),
    /// A scan code appears twice, perhaps written in two widths such as `0x1C` and `0x001C`.
    #[error("scan code {0} appears twice")]
    Duplicate(Scan),
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

/// The file as TOML holds it: scan codes as hex text, plain values before tables.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    id: LayoutId,
    #[serde(default)]
    controls: Vec<String>,
    facts: LangFacts,
    #[serde(default)]
    keys: BTreeMap<String, KeyChars>,
}

/// Reads the control keys, refusing one listed twice.
fn read_controls(texts: &[String]) -> Result<Vec<Scan>, LayoutFileError> {
    let mut controls = Vec::with_capacity(texts.len());
    for text in texts {
        let scan: Scan = text.parse()?;
        if controls.contains(&scan) {
            return Err(LayoutFileError::Duplicate(scan));
        }
        controls.push(scan);
    }
    Ok(controls)
}

/// Reads the key tables, refusing a scan code written twice.
fn read_keys(raw: BTreeMap<String, KeyChars>) -> Result<BTreeMap<Scan, KeyChars>, LayoutFileError> {
    let mut keys = BTreeMap::new();
    for (text, chars) in raw {
        let scan: Scan = text.parse()?;
        if keys.insert(scan, chars).is_some() {
            return Err(LayoutFileError::Duplicate(scan));
        }
    }
    Ok(keys)
}

impl LayoutFile {
    /// Reads a table from TOML text.
    ///
    /// # Errors
    /// `Toml` when the text is not a table, `BadCode` or `Duplicate` for a bad scan code.
    pub fn parse(text: &str) -> Result<Self, LayoutFileError> {
        let raw: Raw = toml::from_str(text)?;
        Ok(Self {
            id: raw.id,
            facts: raw.facts,
            controls: read_controls(&raw.controls)?,
            keys: read_keys(raw.keys)?,
        })
    }

    /// The table as TOML text, with scan codes as four hex digits.
    ///
    /// # Errors
    /// `Write` when TOML cannot hold a value.
    pub fn to_toml(&self) -> Result<String, LayoutFileError> {
        let raw = Raw {
            id: self.id,
            controls: self.controls.iter().map(ToString::to_string).collect(),
            facts: self.facts.clone(),
            keys: self
                .keys
                .iter()
                .map(|(c, k)| (c.to_string(), k.clone()))
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

    /// Reads this crate's golden table `name`, such as `fr-fr`.
    ///
    /// # Errors
    /// As `read`.
    pub fn golden(name: &str) -> Result<Self, LayoutFileError> {
        Self::read(&golden_dir().join(name).with_extension(FILE_EXT))
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
    use proptest::collection::{btree_map, btree_set, vec};
    use proptest::prelude::*;
    use proptest::sample::select;

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
    fn a_code_written_twice_in_two_widths_is_refused() {
        let text = table().to_toml().unwrap();
        let twice = format!("{text}\n[keys.0x1A]\nbase = \"x\"\n");
        let err = LayoutFile::parse(&twice).unwrap_err();
        assert!(
            matches!(err, LayoutFileError::Duplicate(s) if s == Scan::new(0x1A)),
            "{err}"
        );
        let controls = text.replace(r#"["0x001C"]"#, r#"["0x1C", "0x001C"]"#);
        let err = LayoutFile::parse(&controls).unwrap_err();
        assert!(
            matches!(err, LayoutFileError::Duplicate(s) if s == Scan::new(0x1C)),
            "{err}"
        );
    }

    #[test]
    fn a_bad_id_is_refused() {
        let text = table().to_toml().unwrap();
        let broken = text.replace("0xFFFFFFFFF0020409", "1234");
        assert!(LayoutFile::parse(&broken).is_err());
    }

    fn any_chars() -> impl Strategy<Value = KeyChars> {
        let text = || any::<String>();
        let dead = vec(select(Slot::ALL.to_vec()), 0..3);
        (text(), text(), text(), text(), text(), dead).prop_map(|(b, s, a, c, cs, dead)| KeyChars {
            base: b,
            shift: s,
            altgr: a,
            caps: c,
            caps_shift: cs,
            dead,
        })
    }

    fn any_table() -> impl Strategy<Value = LayoutFile> {
        let facts = (
            any::<String>(),
            any::<String>(),
            any::<String>(),
            any::<bool>(),
        );
        let controls = btree_set(any::<u16>(), 0..4);
        let keys = btree_map(any::<u16>(), any_chars(), 0..6);
        (any::<u64>(), facts, controls, keys).prop_map(|(id, (iso, native, layout, rtl), c, k)| {
            LayoutFile {
                id: LayoutId::new(id),
                facts: LangFacts {
                    iso,
                    native,
                    layout,
                    rtl,
                },
                controls: c.into_iter().map(Scan::new).collect(),
                keys: k
                    .into_iter()
                    .map(|(code, chars)| (Scan::new(code), chars))
                    .collect(),
            }
        })
    }

    proptest! {
        #[test]
        fn any_table_round_trips(table in any_table()) {
            let text = table.to_toml().unwrap();
            prop_assert_eq!(LayoutFile::parse(&text).unwrap(), table);
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
