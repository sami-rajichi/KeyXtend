//! The layouts service: the current layout's table, the ordered list and the language switch.

use super::{KeyChars, LayoutId, Scan, SwitchError};
use crate::{Capability, Event, ServiceId, ServiceKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

/// How the keyboard names one layout.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LangLabel {
    /// The layout.
    pub id: LayoutId,
    /// What the language key shows, such as `EN` or `ع`.
    pub short: String,
    /// The space bar's text parts: the language, then the layout when known; the UI joins them.
    pub name: Vec<String>,
    /// The language is written right to left.
    pub rtl: bool,
    /// The labels need the Arabic font.
    pub arabic: bool,
}

/// One layout ready to draw: its label and what each key types.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutTable {
    /// How the keyboard names the layout.
    pub label: LangLabel,
    /// What each key types, by scan code.
    pub keys: BTreeMap<Scan, KeyChars>,
}

/// The layouts the keyboard offers, and the switch between them; `kx-mod-layouts` provides it.
pub trait Layouts: Send + Sync {
    /// The table of the layout the keyboard shows now.
    fn current(&self) -> Arc<LayoutTable>;

    /// The installed layouts, in the order the language key steps through them.
    fn list(&self) -> Vec<LangLabel>;

    /// Reads the app in front and the installed list again; true when anything changed.
    fn refresh(&self) -> bool;

    /// Asks the app in front to switch `by` places along the list, wrapping.
    /// It only asks: `current` changes when a later `refresh` sees the app switch, which may never happen.
    ///
    /// # Errors
    /// The `SwitchError` of the request; the current table stays.
    fn step(&self, by: i32) -> Result<(), SwitchError>;

    /// Asks the app in front to switch to layout `id`; like `step`, it only asks.
    ///
    /// # Errors
    /// The `SwitchError` of the request; the current table stays.
    fn pick(&self, id: LayoutId) -> Result<(), SwitchError>;
}

/// The layouts service; it needs `InjectInput`, since a switch acts on the app in front.
pub struct LayoutsKey;

impl ServiceKey for LayoutsKey {
    type Api = dyn Layouts;
    const ID: ServiceId = ServiceId::new("layouts");
    const CAPABILITY: Option<Capability> = Some(Capability::InjectInput);
}

/// The keyboard now shows layout `id`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LayoutChanged {
    /// The layout shown now.
    pub id: LayoutId,
}

impl Event for LayoutChanged {
    const NAME: &'static str = "layout-changed";
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> LayoutTable {
        let q = KeyChars {
            base: "ض".into(),
            shift: "\u{64E}".into(),
            ..KeyChars::default()
        };
        LayoutTable {
            label: LangLabel {
                id: LayoutId::new(0x0401_0401),
                short: "ع".into(),
                name: vec!["العربية".into(), "101".into()],
                rtl: true,
                arabic: true,
            },
            keys: [(Scan::new(0x10), q)].into_iter().collect(),
        }
    }

    #[test]
    fn a_table_round_trips_through_json_and_toml() {
        let json = serde_json::to_string(&table()).unwrap();
        assert_eq!(serde_json::from_str::<LayoutTable>(&json).unwrap(), table());
        let text = toml::to_string(&table()).unwrap();
        assert_eq!(toml::from_str::<LayoutTable>(&text).unwrap(), table());
    }

    #[test]
    fn the_layouts_service_needs_inject_input() {
        assert_eq!(LayoutsKey::ID.as_str(), "layouts");
        assert_eq!(LayoutsKey::CAPABILITY, Some(Capability::InjectInput));
    }

    #[test]
    fn layout_changed_is_open_to_every_module() {
        assert_eq!(LayoutChanged::NAME, "layout-changed");
        assert_eq!(LayoutChanged::CAPABILITY, None);
    }
}
