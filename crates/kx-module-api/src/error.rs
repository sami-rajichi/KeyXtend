//! Errors a module meets while it starts: services, settings and its own start.

use crate::{Capability, ServiceId};
use thiserror::Error;

/// Why a service could not be got or provided.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum ServiceError {
    /// The manifest does not list the service in `requires`.
    #[error("service {0} is not in the manifest's requires")]
    NotRequired(ServiceId),
    /// The service needs a capability the policy did not grant.
    #[error("service {0} needs the {cap} capability, which is not granted", cap = .1.name())]
    NotGranted(ServiceId, Capability),
    /// No running module provides the service.
    #[error("service {0} has no provider")]
    Missing(ServiceId),
    /// The manifest does not list the service in `provides`.
    #[error("service {0} is not in the manifest's provides")]
    NotDeclared(ServiceId),
    /// Another module already provides the service.
    #[error("service {0} already has a provider")]
    AlreadyProvided(ServiceId),
    /// The stored service is not the type its key names.
    #[error("service {0} is stored with another type")]
    WrongType(ServiceId),
    /// The request names another capability than the one stored with the service.
    #[error("service {0} was requested with another capability than its provider's")]
    CapabilityMismatch(ServiceId),
}

/// Why a settings section could not be read, upgraded or accepted.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum SettingsError {
    /// The section does not fit the settings type.
    #[error("settings do not parse: {0}")]
    Parse(String),
    /// The section parses but breaks one of the module's rules.
    #[error("settings are invalid: {0}")]
    Invalid(String),
    /// The migration from version `from` failed.
    #[error("settings migration from version {from} failed: {message}")]
    Migration {
        /// The version the failed migration started from.
        from: u32,
        /// What went wrong.
        message: String,
    },
}

/// Why a module failed to start; the kernel marks it `Failed` and keeps the rest running.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ModuleError {
    /// A service could not be got or provided.
    #[error(transparent)]
    Service(#[from] ServiceError),
    /// The module's settings were unusable.
    #[error(transparent)]
    Settings(#[from] SettingsError),
    /// The module's own start failed; the reason is a short fixed text and never holds user data.
    #[error("start failed: {0}")]
    Start(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIC: ServiceId = ServiceId::new("mic");

    fn fail_service() -> Result<(), ModuleError> {
        Err(ServiceError::Missing(MIC))?
    }

    fn fail_settings() -> Result<(), ModuleError> {
        Err(SettingsError::Parse("bad".into()))?
    }

    #[test]
    fn question_mark_wraps_service_and_settings_errors() {
        assert_eq!(
            fail_service(),
            Err(ModuleError::Service(ServiceError::Missing(MIC)))
        );
        let parse = SettingsError::Parse("bad".into());
        assert_eq!(fail_settings(), Err(ModuleError::Settings(parse)));
    }

    #[test]
    fn not_granted_names_the_capability() {
        let err = ServiceError::NotGranted(MIC, Capability::Microphone);
        let text = err.to_string();
        assert!(
            text.contains("mic") && text.contains(Capability::Microphone.name()),
            "{text}"
        );
    }
}
