//! Reading the OS keyboard layouts: what each key types, the language facts and the switch request.

use super::Scan;
use crate::{Capability, ServiceId, ServiceKey};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// An opaque OS keyboard layout handle; data files and JSON hold it as `0x` hex text.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayoutId(u64);

impl LayoutId {
    /// Wraps the adapter's raw handle.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// The raw handle.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A modifier state a key is read in.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Slot {
    /// No modifier.
    Base,
    /// Shift.
    Shift,
    /// `AltGr`, named as its `KeyChars` field.
    #[serde(rename = "altgr")]
    AltGr,
    /// Caps Lock on.
    Caps,
    /// Caps Lock on, with Shift.
    CapsShift,
}

impl Slot {
    /// Every slot, in the order of `KeyChars`' fields.
    pub const ALL: [Self; 5] = [
        Self::Base,
        Self::Shift,
        Self::AltGr,
        Self::Caps,
        Self::CapsShift,
    ];
}

/// What one key types in one layout, per modifier state; empty when it types nothing.
/// Empty fields are left out of data files and read back as empty.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct KeyChars {
    /// No modifier.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub base: String,
    /// Shift.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub shift: String,
    /// `AltGr`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub altgr: String,
    /// Caps Lock on, as the layout gives it.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub caps: String,
    /// Caps Lock on, with Shift.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub caps_shift: String,
    /// The states whose character is a dead key, which waits for the next key to add its accent.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dead: Vec<Slot>,
}

impl KeyChars {
    /// The text in state `s`.
    #[must_use]
    pub fn get(&self, s: Slot) -> &str {
        match s {
            Slot::Base => &self.base,
            Slot::Shift => &self.shift,
            Slot::AltGr => &self.altgr,
            Slot::Caps => &self.caps,
            Slot::CapsShift => &self.caps_shift,
        }
    }

    /// True when state `s` is a dead key, so the app must see a real key press.
    #[must_use]
    pub fn is_dead(&self, s: Slot) -> bool {
        self.dead.contains(&s)
    }
}

/// What the OS says about a layout's language.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LangFacts {
    /// The ISO 639 language code, such as `ar`.
    pub iso: String,
    /// The language's name in itself, such as `العربية`.
    pub native: String,
    /// The layout's own name, such as `Arabic (101)`.
    pub layout: String,
    /// The language is written right to left.
    pub rtl: bool,
}

/// Why the app in front could not be asked to switch layout.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum SwitchError {
    /// No app window is in front to ask.
    #[error("no app window is in front")]
    NoTarget,
    /// The layout is not installed.
    #[error("the layout is not installed")]
    NotInstalled,
    /// The OS refused the request.
    #[error("the system refused the layout request")]
    Refused,
}

/// Reads the installed layouts and asks the app in front to switch.
pub trait LayoutSource: Send + Sync {
    /// The installed layouts, in the OS order.
    fn installed(&self) -> Vec<LayoutId>;

    /// The layout of the app in front, when there is one.
    fn foreground(&self) -> Option<LayoutId>;

    /// What each of `scans` types in `layout`, in the same order; a key that types nothing is empty.
    fn keys(&self, layout: LayoutId, scans: &[Scan]) -> Vec<KeyChars>;

    /// The language facts of `layout`; empty when the OS has none.
    fn facts(&self, layout: LayoutId) -> LangFacts;

    /// True while Caps Lock is on.
    fn caps_on(&self) -> bool;

    /// True when `key` types anything in `layout` with `held` down, even a control such as Enter's.
    fn types_any(&self, layout: LayoutId, key: Scan, held: &[Scan]) -> bool;

    /// Asks the app in front to switch to `layout`; the app switches later or never, as `foreground` shows.
    ///
    /// # Errors
    /// `NoTarget` with no app in front, `NotInstalled` for an unknown layout, `Refused` when the OS says no.
    fn request(&self, layout: LayoutId) -> Result<(), SwitchError>;
}

/// The layout source service; it needs `InjectInput`, since it acts on the app in front.
pub struct LayoutSourceKey;

impl ServiceKey for LayoutSourceKey {
    type Api = dyn LayoutSource;
    const ID: ServiceId = ServiceId::new("layout-source");
    const CAPABILITY: Option<Capability> = Some(Capability::InjectInput);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn every_slot() -> KeyChars {
        KeyChars {
            base: "é".into(),
            shift: "2".into(),
            altgr: "~".into(),
            caps: "É".into(),
            caps_shift: "2̃".into(),
            dead: vec![Slot::AltGr],
        }
    }

    #[test]
    fn get_returns_each_slot() {
        let k = every_slot();
        let want = [&k.base, &k.shift, &k.altgr, &k.caps, &k.caps_shift];
        for (slot, text) in Slot::ALL.into_iter().zip(want) {
            assert_eq!(k.get(slot), text, "{slot:?}");
        }
    }

    #[test]
    fn only_the_listed_slots_are_dead() {
        let k = every_slot();
        let dead: Vec<Slot> = Slot::ALL.into_iter().filter(|&s| k.is_dead(s)).collect();
        assert_eq!(dead, [Slot::AltGr]);
    }

    #[test]
    fn key_chars_round_trip_through_toml_and_json() {
        let k = every_slot();
        let toml_text = toml::to_string(&k).unwrap();
        assert_eq!(toml::from_str::<KeyChars>(&toml_text).unwrap(), k);
        let json = serde_json::to_string(&k).unwrap();
        assert_eq!(serde_json::from_str::<KeyChars>(&json).unwrap(), k);
    }

    #[test]
    fn slots_use_their_field_names() {
        let text = serde_json::to_string(&Slot::ALL).unwrap();
        assert_eq!(text, r#"["base","shift","altgr","caps","caps_shift"]"#);
    }

    #[test]
    fn empty_fields_are_left_out_and_unknown_ones_are_refused() {
        let text = "base = \"a\"\n";
        let k: KeyChars = toml::from_str(text).unwrap();
        let want = KeyChars {
            base: "a".into(),
            ..KeyChars::default()
        };
        assert_eq!(k, want);
        assert_eq!(toml::to_string(&want).unwrap(), text);
        assert!(toml::from_str::<KeyChars>("alt_gr = \"€\"").is_err());
    }

    #[test]
    fn facts_round_trip() {
        let facts = LangFacts {
            iso: "ar".into(),
            native: "العربية".into(),
            layout: "Arabic (101)".into(),
            rtl: true,
        };
        let text = toml::to_string(&facts).unwrap();
        assert_eq!(toml::from_str::<LangFacts>(&text).unwrap(), facts);
    }

    #[test]
    fn a_layout_id_keeps_its_raw_value() {
        assert_eq!(LayoutId::new(u64::MAX).get(), u64::MAX);
    }

    #[test]
    fn the_layout_source_needs_inject_input() {
        assert_eq!(LayoutSourceKey::ID.as_str(), "layout-source");
        assert_eq!(LayoutSourceKey::CAPABILITY, Some(Capability::InjectInput));
    }
}
