//! The contract between the kernel and feature modules: ids, manifests, services, events,
//! settings, notices, time and redaction.
//! Modules depend on this crate and on nothing else in the workspace.

#![forbid(unsafe_code)]

mod capability;
mod clock;
mod cx;
mod error;
mod event;
mod ids;
mod manifest;
mod notice;
mod redacted;
mod service;
mod settings;

pub use capability::Capability;
pub use clock::{Clock, MS_PER_S, Mono, US_PER_MS, US_PER_S};
pub use cx::{Host, ModuleCx};
pub use error::{ModuleError, ServiceError, SettingsError};
pub use event::{Bus, BusCore, Event, Flow, InterceptFn, NotifyFn, Subscription, SubscriptionId};
pub use ids::{ModuleId, ServiceId};
pub use manifest::{Manifest, Module};
pub use notice::{ModuleState, ModuleStateChanged, Notice, SettingsChanged, keys};
pub use redacted::{REDACTED_MARKER, Redacted};
pub use service::ServiceKey;
pub use settings::{Migration, Settings, SettingsSpec, parse_as, validate_as};
