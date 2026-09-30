//! The `[layouts]` settings: what the language key shows for each language.

use kx_module_api::{Settings, SettingsError, SettingsSpec, parse_as, validate_as};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Why a `short` key was refused.
const BAD_CODE: &str = "short: every key must be an ISO 639 code in lower-case letters";
/// Why a `short` value was refused.
const BLANK: &str = "short: a short name is blank";

/// How the kernel reads the `[layouts]` section: version 1, with the defaults embedded.
pub static SPEC: SettingsSpec = SettingsSpec {
    version: 1,
    defaults: include_str!("../defaults.toml"),
    migrations: &[],
    validate: validate_as::<LayoutsSettings>,
};

/// The module's settings; every value comes from `defaults.toml` or the user's file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutsSettings {
    /// What the language key shows, by ISO 639 code, such as `ar = "ع"`; others show the code in capitals.
    pub short: BTreeMap<String, String>,
}

impl LayoutsSettings {
    /// The settings in the embedded `defaults.toml`.
    ///
    /// # Errors
    /// `SettingsError::Parse` when the defaults do not parse, or the error from `check`.
    pub fn from_defaults() -> Result<Self, SettingsError> {
        let table: toml::Table = SPEC
            .defaults
            .parse()
            .map_err(|e: toml::de::Error| SettingsError::Parse(e.to_string()))?;
        parse_as(&table)
    }
}

impl Settings for LayoutsSettings {
    fn check(&self) -> Result<(), SettingsError> {
        let invalid = |why: &str| Err(SettingsError::Invalid(why.to_owned()));
        if !self.short.keys().all(|k| is_iso_code(k)) {
            return invalid(BAD_CODE);
        }
        if self.short.values().any(|v| v.trim().is_empty()) {
            return invalid(BLANK);
        }
        Ok(())
    }
}

/// True for an ISO 639 code as the OS gives it: lower-case ASCII letters only.
fn is_iso_code(code: &str) -> bool {
    !code.is_empty() && code.bytes().all(|b| b.is_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(text: &str) -> toml::Table {
        text.parse().unwrap()
    }

    fn with_short(pairs: &[(&str, &str)]) -> toml::Table {
        let short: toml::Table = pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), toml::Value::from(*v)))
            .collect();
        toml::Table::from_iter([("short".to_owned(), toml::Value::Table(short))])
    }

    fn invalid(t: &toml::Table) -> bool {
        matches!(
            validate_as::<LayoutsSettings>(t),
            Err(SettingsError::Invalid(_))
        )
    }

    #[test]
    fn the_defaults_validate_and_name_arabic_by_its_letter() {
        assert!(SPEC.is_consistent());
        assert_eq!((SPEC.validate)(&table(SPEC.defaults)), Ok(()));
        let s = LayoutsSettings::from_defaults().unwrap();
        assert_eq!(s.short.get("ar").map(String::as_str), Some("ع"));
    }

    #[test]
    fn a_short_key_must_be_a_lower_case_iso_code() {
        assert!(!invalid(&with_short(&[("fa", "ف"), ("haw", "HAW")])));
        for bad in ["", "AR", "Ar", "a1", "ar-tn", " ar", "ع"] {
            assert!(invalid(&with_short(&[(bad, "x")])), "{bad:?}");
        }
    }

    #[test]
    fn a_blank_short_name_is_refused() {
        for blank in ["", " ", "\t"] {
            assert!(invalid(&with_short(&[("ar", blank)])), "{blank:?}");
        }
    }

    #[test]
    fn a_missing_or_unknown_key_does_not_parse() {
        let parse = |t: &toml::Table| {
            matches!(
                validate_as::<LayoutsSettings>(t),
                Err(SettingsError::Parse(_))
            )
        };
        assert!(parse(&table("")), "short has no default in code");
        assert!(parse(&table("short = {}\njoiner = \" · \"")));
    }
}
