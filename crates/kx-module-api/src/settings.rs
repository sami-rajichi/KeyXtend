//! A module's settings contract: section version, defaults, migrations and validator.

use crate::SettingsError;
use serde::{Serialize, de::DeserializeOwned};

/// A module's settings type: read with serde, plus rules serde cannot express.
///
/// Mark the type `#[serde(deny_unknown_fields)]`, or a misspelt key passes silently.
pub trait Settings: Serialize + DeserializeOwned {
    /// Checks rules such as ranges; the default accepts everything.
    ///
    /// # Errors
    /// `SettingsError::Invalid` when a rule is broken.
    fn check(&self) -> Result<(), SettingsError> {
        Ok(())
    }
}

/// Upgrades a settings section by one version, in place; it sees the values without the `version` key.
pub type Migration = fn(&mut toml::Table) -> Result<(), SettingsError>;

/// How the kernel reads, upgrades and checks one module's settings section.
/// Sections are flat: a nested table is compared and reset as one value.
#[derive(Debug)]
pub struct SettingsSpec {
    /// The current section version, starting at 1.
    pub version: u32,
    /// The module's embedded `defaults.toml` text: every setting, and no `version` key.
    /// A key in the user's file that is not in it is reset, whatever `validate` says.
    pub defaults: &'static str,
    /// Entry `i` upgrades version `i + 1` to `i + 2`.
    pub migrations: &'static [Migration],
    /// Checks a whole section, usually `validate_as::<T>`.
    pub validate: fn(&toml::Table) -> Result<(), SettingsError>,
}

impl SettingsSpec {
    /// True when there is one migration per version step, `version - 1` in all.
    #[must_use]
    pub fn is_consistent(&self) -> bool {
        let steps = self
            .version
            .checked_sub(1)
            .and_then(|n| usize::try_from(n).ok());
        steps == Some(self.migrations.len())
    }
}

/// Checks `table` by reading it as `T` and running `T::check`.
///
/// # Errors
/// `SettingsError::Parse` when it does not fit `T`, or the error from `check`.
pub fn validate_as<T: Settings>(table: &toml::Table) -> Result<(), SettingsError> {
    parse_as::<T>(table).map(drop)
}

/// Reads `table` as `T` and runs `T::check`.
///
/// # Errors
/// `SettingsError::Parse` when it does not fit `T`, or the error from `check`.
pub fn parse_as<T: Settings>(table: &toml::Table) -> Result<T, SettingsError> {
    let value = T::deserialize(table.clone()).map_err(|e| SettingsError::Parse(e.to_string()))?;
    value.check()?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    const MAX_SPEED: u32 = 10;

    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Demo {
        speed: u32,
    }

    impl Settings for Demo {
        fn check(&self) -> Result<(), SettingsError> {
            if self.speed > MAX_SPEED {
                return Err(SettingsError::Invalid("speed is too high".into()));
            }
            Ok(())
        }
    }

    #[derive(Serialize, Deserialize)]
    struct Plain {
        speed: u32,
    }

    impl Settings for Plain {}

    fn table(text: &str) -> toml::Table {
        text.parse().unwrap()
    }

    #[allow(clippy::unnecessary_wraps, reason = "matches the Migration signature")]
    fn step(_: &mut toml::Table) -> Result<(), SettingsError> {
        Ok(())
    }

    fn spec(version: u32, migrations: &'static [Migration]) -> SettingsSpec {
        SettingsSpec {
            version,
            defaults: "speed = 3",
            migrations,
            validate: validate_as::<Demo>,
        }
    }

    #[test]
    fn one_migration_per_step_is_consistent() {
        assert!(spec(1, &[]).is_consistent());
        assert!(spec(3, &[step, step]).is_consistent());
    }

    #[test]
    fn a_missing_or_extra_migration_is_inconsistent() {
        assert!(!spec(2, &[]).is_consistent());
        assert!(!spec(1, &[step]).is_consistent());
    }

    #[test]
    fn version_zero_is_inconsistent() {
        assert!(!spec(0, &[]).is_consistent());
    }

    #[test]
    fn an_unknown_key_fails_to_parse() {
        let bad = table("speed = 3\nextra = 1");
        assert!(matches!(
            validate_as::<Demo>(&bad),
            Err(SettingsError::Parse(_))
        ));
        assert!(matches!(
            parse_as::<Demo>(&bad),
            Err(SettingsError::Parse(_))
        ));
    }

    #[test]
    fn a_failing_check_is_invalid() {
        let fast = table(&format!("speed = {}", MAX_SPEED + 1));
        assert!(matches!(
            validate_as::<Demo>(&fast),
            Err(SettingsError::Invalid(_))
        ));
        assert!(matches!(
            parse_as::<Demo>(&fast),
            Err(SettingsError::Invalid(_))
        ));
    }

    #[test]
    fn a_good_table_parses() {
        let good = table("speed = 3");
        assert_eq!(validate_as::<Demo>(&good), Ok(()));
        assert_eq!(parse_as::<Demo>(&good).map(|d| d.speed), Ok(3));
    }

    #[test]
    fn the_default_check_accepts() {
        assert_eq!(
            parse_as::<Plain>(&table("speed = 99")).map(|p| p.speed),
            Ok(99)
        );
    }

    #[test]
    fn the_spec_validates_its_defaults() {
        let spec = spec(1, &[]);
        assert_eq!((spec.validate)(&table(spec.defaults)), Ok(()));
    }
}
