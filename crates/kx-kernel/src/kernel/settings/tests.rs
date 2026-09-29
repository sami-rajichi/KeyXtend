//! The kernel's own settings: the level names, the log size range and the disabled list.

use super::*;
use crate::log::LOG_LEVELS;

/// Kernel settings that pass `check`.
fn sample() -> KernelSettings {
    KernelSettings {
        disabled: Vec::new(),
        log_level: LOG_LEVELS[0].to_owned(),
        log_max_mb: MIN_LOG_MB,
    }
}

#[test]
fn the_kernel_settings_take_their_level_names_from_the_log_list() {
    let with = |name: &str| KernelSettings {
        log_level: name.to_owned(),
        ..sample()
    };
    for name in LOG_LEVELS {
        assert!(with(name).check().is_ok(), "{name}");
    }
    assert!(with("loud").check().is_err());
}

#[test]
fn the_kernel_settings_cap_the_log_size() {
    let with = |mb: u32| KernelSettings {
        log_max_mb: mb,
        ..sample()
    };
    assert!(with(MIN_LOG_MB).check().is_ok());
    assert!(with(MAX_LOG_MB).check().is_ok());
    assert!(with(MIN_LOG_MB - 1).check().is_err());
    assert!(with(MAX_LOG_MB + 1).check().is_err());
}

#[test]
fn the_kernel_settings_refuse_an_empty_id_in_the_disabled_list() {
    let with = |ids: &[&str]| KernelSettings {
        disabled: ids.iter().map(|id| (*id).to_owned()).collect(),
        ..sample()
    };
    assert!(with(&["a", "b"]).check().is_ok());
    assert!(with(&["a", ""]).check().is_err());
}
