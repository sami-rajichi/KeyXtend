//! Shortcuts by meaning, such as `copy-all`: the platform knows each id's keys and sends them, held latches and all.
//! Ids and key names can show what the user does, so no `Debug` here prints one.

use super::Scan;
use crate::{Capability, REDACTED_MARKER, ServiceId, ServiceKey};
use serde::{Deserialize, Deserializer, Serialize, de};
use std::fmt;
use thiserror::Error;

/// The longest shortcut id or key name, in bytes.
pub const MAX_ID_LEN: usize = 32;

/// What a refused id or name hears; it never repeats the text, which may be a key.
const BAD_FORM: &str = "not lower-case letters, digits and inner dashes within the length limit";

/// True for lower-case ASCII letters, digits and `-`, not at either end, and at most `MAX_ID_LEN` long.
fn well_formed(text: &str) -> bool {
    let plain = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-';
    let edges = !text.starts_with('-') && !text.ends_with('-');
    !text.is_empty() && text.len() <= MAX_ID_LEN && edges && text.bytes().all(plain)
}

/// Declares a checked name over `String`: `new`, `as_str`, a redacted `Debug` and serde as plain text that refuses
/// a bad form.
macro_rules! checked_name {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Wraps `text`, or `None` when it is not lower-case letters, digits and inner dashes up to `MAX_ID_LEN`.
            #[must_use]
            pub fn new(text: &str) -> Option<Self> {
                well_formed(text).then(|| Self(text.to_owned()))
            }

            /// The text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(REDACTED_MARKER)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let text = String::deserialize(deserializer)?;
                Self::new(&text).ok_or_else(|| de::Error::custom(BAD_FORM))
            }
        }
    };
}

checked_name!(
    /// A shortcut by meaning, such as `copy-all` or `f5`; the platform maps it to keys.
    ShortcutId
);

checked_name!(
    /// A key by a neutral name, such as `ctrl`, `a`, `f5` or `numplus`; the platform maps it to its key.
    KeyName
);

/// Keys held in order around a tap of `key`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    /// Keys pressed first, in order, and let go after the tap.
    pub held: Vec<KeyName>,
    /// The key tapped.
    pub key: KeyName,
}

/// What the platform sends for one id, which a tip also names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortcutInfo {
    /// The shortcut.
    pub id: ShortcutId,
    /// Its steps, sent in order.
    pub steps: Vec<Step>,
}

/// Why a shortcut did not all reach the app in front.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum ShortcutError {
    /// The platform has no such id; nothing was sent.
    #[error("the system has no such shortcut")]
    Unknown,
    /// The OS took only `sent` of `total` events; held keys were let go.
    #[error("the system took {sent} of {total} shortcut events")]
    Refused {
        /// Events the OS took.
        sent: usize,
        /// Events that were sent.
        total: usize,
    },
    /// An OS call, such as the session lock, failed.
    #[error("the system call behind the shortcut failed")]
    Failed,
}

/// Sends shortcuts by id to the app in front; the platform provides it.
pub trait Shortcuts: Send + Sync {
    /// Every id this platform can send, with its steps.
    fn known(&self) -> Vec<ShortcutInfo>;

    /// Sends every step of `id` with the latched `held` keys pressed around each step; a key held twice is pressed
    /// once.
    ///
    /// # Errors
    /// `Unknown` for an id not in `known`, `Refused` when the OS drops events, `Failed` when an OS call fails.
    fn send(&self, id: &ShortcutId, held: &[Scan]) -> Result<(), ShortcutError>;
}

/// The shortcuts service; it needs `InjectInput`.
pub struct ShortcutsKey;

impl ServiceKey for ShortcutsKey {
    type Api = dyn Shortcuts;
    const ID: ServiceId = ServiceId::new("shortcuts");
    const CAPABILITY: Option<Capability> = Some(Capability::InjectInput);
}

#[cfg(test)]
mod tests;
