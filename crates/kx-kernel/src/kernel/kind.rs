//! A short fixed name for each `KernelError`, safe to log where the error's own text may quote a settings value.

use super::KernelError;
use kx_settings::StoreError;

/// The log name of a `KernelError::Unknown`.
const UNKNOWN: &str = "module not added";
/// The log name of a `KernelError::Duplicate`.
const DUPLICATE: &str = "module already added";
/// The log name of a `KernelError::Reserved`.
const RESERVED: &str = "the kernel's own id";
/// The log name of a `KernelError::NotFailed`.
const NOT_FAILED: &str = "module has not failed";
/// The log name of a `KernelError::NotStartable`.
const NOT_STARTABLE: &str = "module cannot start";
/// The log name of a `KernelError::WrongPhase`.
const WRONG_PHASE: &str = "wrong kernel phase";
/// The log name of a `KernelError::SwitchKey`.
const SWITCH_KEY: &str = "disabled list changes only by switching";
/// The log name of a `KernelError::Service`.
const SERVICE: &str = "service refused";
/// The log name of a `KernelError::Settings` that is not a refused value.
const SETTINGS: &str = "settings change refused";

impl KernelError {
    /// A short fixed name of the error's kind, safe to log: a refused settings value gives its `SettingsError::kind`.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Unknown(_) => UNKNOWN,
            Self::Duplicate(_) => DUPLICATE,
            Self::Reserved(_) => RESERVED,
            Self::NotFailed(_) => NOT_FAILED,
            Self::NotStartable(_) => NOT_STARTABLE,
            Self::WrongPhase(_) => WRONG_PHASE,
            Self::SwitchKey => SWITCH_KEY,
            Self::Service(_) => SERVICE,
            Self::Settings(StoreError::Settings(e)) => e.kind(),
            Self::Settings(_) => SETTINGS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Phase;
    use kx_module_api::{ModuleId, ServiceError, ServiceId, SettingsError};
    use std::collections::BTreeSet;

    #[test]
    fn a_refused_value_logs_its_settings_kind_and_never_the_value() {
        let quoted = SettingsError::Parse("invalid type: string \"hunter2\"".into());
        let error = KernelError::Settings(StoreError::Settings(quoted.clone()));
        assert!(error.to_string().contains("hunter2"), "the text quotes it");
        assert_eq!(error.kind(), quoted.kind());
    }

    #[test]
    fn each_error_has_its_own_fixed_kind() {
        let id = ModuleId::new("a");
        let errors = [
            KernelError::Unknown(id),
            KernelError::Duplicate(id),
            KernelError::Reserved(id),
            KernelError::NotFailed(id),
            KernelError::NotStartable(id),
            KernelError::WrongPhase(Phase::Setup),
            KernelError::SwitchKey,
            KernelError::Service(ServiceError::Missing(ServiceId::new("s"))),
            KernelError::Settings(StoreError::Reserved),
        ];
        let kinds: BTreeSet<&str> = errors.iter().map(KernelError::kind).collect();
        assert_eq!(kinds.len(), errors.len(), "{kinds:?}");
        assert!(!kinds.contains(""));
    }
}
