//! Sending keys and text to the app in front.

use crate::{Capability, Redacted, ServiceId, ServiceKey};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The prefix of an extended key's scan code (`0xE0xx`).
const EXTENDED: u16 = 0xE000;
/// The prefix byte of a scan code.
const PREFIX_MASK: u16 = 0xFF00;

/// A key's scan code; `0xE0xx` marks an extended key. Data files hold it as a plain number.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Scan(u16);

impl Scan {
    /// Wraps a scan code, such as `0x1E` or `0xE035`.
    #[must_use]
    pub const fn new(code: u16) -> Self {
        Self(code)
    }

    /// The full code, with any prefix.
    #[must_use]
    pub const fn code(self) -> u16 {
        self.0
    }

    /// True for an extended key (`0xE0xx`).
    #[must_use]
    pub const fn is_extended(self) -> bool {
        self.0 & PREFIX_MASK == EXTENDED
    }

    /// The code without its prefix byte, as the OS takes it.
    #[must_use]
    pub const fn byte(self) -> u8 {
        self.0.to_le_bytes()[0]
    }
}

/// How text reaches the app in front.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum TextRoute {
    /// As Unicode characters, whatever the target's layout.
    #[default]
    Unicode,
    /// As key presses in the target's layout, for apps that drop Unicode input.
    Keys,
}

/// Why input did not reach the app in front.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum InjectError {
    /// The OS took only `sent` of `total` events.
    #[error("the system took {sent} of {total} input events")]
    Refused {
        /// Events the OS took.
        sent: usize,
        /// Events that were sent.
        total: usize,
    },
    /// A character has no key in the target's layout; nothing was sent.
    #[error("a character has no key in the target layout")]
    Unmappable,
}

/// Sends key presses and text to the app in front.
pub trait InputInjector: Send + Sync {
    /// Presses and releases `key`.
    ///
    /// # Errors
    /// `Refused` when the OS drops events.
    fn tap(&self, key: Scan) -> Result<(), InjectError>;

    /// Presses `held` in order, taps `key`, then releases `held` in reverse.
    ///
    /// # Errors
    /// `Refused` when the OS drops events; held keys are still released.
    fn chord(&self, held: &[Scan], key: Scan) -> Result<(), InjectError>;

    /// Types `text` by `route`.
    ///
    /// # Errors
    /// `Refused` when the OS drops events, `Unmappable` when the Keys route has no key for a character.
    fn text(&self, text: &Redacted<String>, route: TextRoute) -> Result<(), InjectError>;

    /// The file name of the app in front, such as `notepad.exe`, for the `key_apps` setting.
    fn target_app(&self) -> Option<String>;
}

/// The input injector service; it needs `InjectInput`.
pub struct InputInjectorKey;

impl ServiceKey for InputInjectorKey {
    type Api = dyn InputInjector;
    const ID: ServiceId = ServiceId::new("input-injector");
    const CAPABILITY: Option<Capability> = Some(Capability::InjectInput);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The only prefix byte that marks an extended key.
    const EXTENDED_BYTE: u8 = 0xE0;

    #[test]
    fn every_code_splits_into_prefix_and_byte() {
        for code in 0..=u16::MAX {
            let scan = Scan::new(code);
            let [low, high] = code.to_le_bytes();
            assert_eq!(scan.code(), code);
            assert_eq!(scan.byte(), low, "{code:#06X}");
            assert_eq!(scan.is_extended(), high == EXTENDED_BYTE, "{code:#06X}");
        }
    }

    #[test]
    fn extended_and_plain_codes_match_the_spike() {
        assert!(Scan::new(0xE01D).is_extended());
        assert!(!Scan::new(0x1D).is_extended());
        assert!(!Scan::new(0xE11D).is_extended());
        assert_eq!(Scan::new(0xE035).byte(), 0x35);
        assert_eq!(Scan::new(0x2A).byte(), 0x2A);
    }

    #[derive(Deserialize)]
    struct Board {
        key: Scan,
    }

    #[test]
    fn a_scan_is_a_plain_number_in_data_files() {
        let board: Board = toml::from_str("key = 0xE035").unwrap();
        assert_eq!(board.key, Scan::new(0xE035));
        assert_eq!(serde_json::to_string(&Scan::new(0x1E)).unwrap(), "30");
    }

    #[test]
    fn text_goes_as_unicode_by_default() {
        assert_eq!(TextRoute::default(), TextRoute::Unicode);
    }

    #[test]
    fn the_injector_needs_inject_input() {
        assert_eq!(InputInjectorKey::ID.as_str(), "input-injector");
        assert_eq!(InputInjectorKey::CAPABILITY, Some(Capability::InjectInput));
    }

    #[test]
    fn a_refusal_says_how_many_events_went() {
        let text = InjectError::Refused { sent: 3, total: 8 }.to_string();
        assert!(text.contains('3') && text.contains('8'), "{text}");
    }
}
