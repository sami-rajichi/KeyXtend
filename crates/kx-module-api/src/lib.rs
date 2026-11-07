//! The contract between the kernel and feature modules: ids, manifests, services, events, settings, notices, time,
//! redaction, hidden marks, the script blocks, the ports (service traits) in `ports`, folding in `text` and a panel
//! line's caret in `caret`. Modules depend on this crate and on nothing else in the workspace.

#![forbid(unsafe_code)]

mod capability;
pub mod caret;
mod clock;
mod cx;
mod error;
mod event;
mod handle;
mod ids;
mod manifest;
mod marks;
mod notice;
pub mod ports;
mod redacted;
mod script;
mod service;
mod settings;
pub mod text;

pub use capability::Capability;
pub use clock::{Clock, MS_PER_S, Mono, US_PER_MS, US_PER_S};
pub use cx::{Host, ModuleCx};
pub use error::{EventError, ModuleError, ServiceError, SettingsError, WriteError};
pub use event::{Bus, BusCore, Event, Flow, InterceptFn, NotifyFn, Subscription, SubscriptionId};
pub use handle::{SettingsHandle, SettingsSink};
pub use ids::{ModuleId, ServiceId};
pub use manifest::{Manifest, Module};
pub use marks::{HIDDEN_MARKS, INVISIBLE, LINE_BREAKS, is_hidden};
pub use notice::{
    ModuleState, ModuleStateChanged, Notice, NoticeAnswered, NoticeArg, NoticeAsk, SettingsChanged,
    args, keys,
};
pub use redacted::{REDACTED_MARKER, Redacted, Secret, SecretBytes};
pub use script::{ARABIC_BLOCKS, RTL_BLOCKS, is_arabic, is_rtl};
pub use service::ServiceKey;
pub use settings::{Migration, Settings, SettingsSpec, parse_as, refuse_if, validate_as, within};
/// The wiping wrapper inside `Secret`, so a module makes one without naming `zeroize` itself.
pub use zeroize::Zeroizing;
