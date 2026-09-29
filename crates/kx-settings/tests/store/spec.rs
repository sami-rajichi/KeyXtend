//! The sample module: a hold time pair with a cross-field rule, three versions and two migrations.

#![allow(
    clippy::unwrap_used,
    reason = "these fixture helpers run only in tests, where a failed setup should fail the test"
)]

use kx_module_api::{ModuleId, Notice, Settings, SettingsError, SettingsSpec, keys, validate_as};
use kx_settings::{ARG_KEYS, ARG_MODULE, Files, LoadReport, Store};
use kx_test_support::tempdir::TempDir;
use serde::{Deserialize, Serialize};
use std::fs;
use toml::{Table, Value};

/// The sample module.
pub const HOLD: ModuleId = ModuleId::new("hold");
/// Its current version.
pub const VERSION: u32 = 3;
/// Its defaults.
pub const DEFAULTS: &str = "min_ms = 100\nmax_ms = 500\nsound = true\n";
/// Version 1 kept the delay in seconds under this key.
pub const V1_DELAY: &str = "delay_s";
/// Version 2 kept it in milliseconds under this key.
pub const V2_DELAY: &str = "delay_ms";
/// Version 3 calls it the minimum hold time.
pub const V3_DELAY: &str = "min_ms";
/// Milliseconds per second, for the first migration.
const MS_PER_S: i64 = 1000;
/// The widest gap allowed between the minimum and the maximum.
const MAX_SPAN_MS: u32 = 5000;

/// The sample settings; the minimum may not pass the maximum, nor sit more than `MAX_SPAN_MS` below it.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hold {
    pub min_ms: u32,
    pub max_ms: u32,
    pub sound: bool,
}

impl Settings for Hold {
    fn check(&self) -> Result<(), SettingsError> {
        if self.min_ms > self.max_ms {
            return Err(SettingsError::Invalid("min_ms is above max_ms".into()));
        }
        if self.max_ms - self.min_ms > MAX_SPAN_MS {
            return Err(SettingsError::Invalid("the hold range is too wide".into()));
        }
        Ok(())
    }
}

/// 1 to 2: the delay moves from seconds to milliseconds; a delay that is not whole seconds fails.
fn to_v2(t: &mut Table) -> Result<(), SettingsError> {
    let Some(delay) = t.remove(V1_DELAY) else {
        return Ok(());
    };
    let ms = delay.as_integer().and_then(|s| s.checked_mul(MS_PER_S));
    let ms = ms.ok_or_else(|| SettingsError::Migration {
        from: 1,
        message: "the delay is not whole seconds".into(),
    })?;
    t.insert(V2_DELAY.into(), Value::Integer(ms));
    Ok(())
}

/// 2 to 3: the delay becomes the minimum hold time.
#[allow(clippy::unnecessary_wraps, reason = "matches the Migration signature")]
fn to_v3(t: &mut Table) -> Result<(), SettingsError> {
    if let Some(delay) = t.remove(V2_DELAY) {
        t.insert(V3_DELAY.into(), delay);
    }
    Ok(())
}

/// The sample module's spec.
pub static SPEC: SettingsSpec = SettingsSpec {
    version: VERSION,
    defaults: DEFAULTS,
    migrations: &[to_v2, to_v3],
    validate: validate_as::<Hold>,
};

/// A temp data folder and its file names.
pub fn folder(label: &str) -> (TempDir, Files) {
    let dir = TempDir::new(label).unwrap();
    let files = Files::new(dir.path());
    (dir, files)
}

/// A temp data folder whose settings file holds `text`.
pub fn with_file(label: &str, text: &str) -> (TempDir, Files) {
    let (dir, files) = folder(label);
    fs::write(&files.settings, text).unwrap();
    (dir, files)
}

/// Loads with only the sample module registered.
pub fn open(files: &Files) -> (Store, LoadReport) {
    Store::load(files.clone(), &[(HOLD, &SPEC)])
}

pub fn table(text: &str) -> Table {
    text.parse().unwrap()
}

/// The sample module's defaults with `text` laid over them.
pub fn defaults_with(text: &str) -> Table {
    let mut all = table(DEFAULTS);
    all.extend(table(text));
    all
}

/// The notice for sample-module values that went back to their defaults.
pub fn reset(keys_text: &str) -> Notice {
    Notice {
        key: keys::SETTINGS_RESET,
        module: Some(HOLD),
        args: vec![(ARG_MODULE, HOLD.to_string()), (ARG_KEYS, keys_text.into())],
    }
}

/// The notice for a sample-module section from a newer version.
pub fn newer() -> Notice {
    Notice {
        key: keys::SETTINGS_NEWER,
        module: Some(HOLD),
        args: vec![(ARG_MODULE, HOLD.to_string())],
    }
}
