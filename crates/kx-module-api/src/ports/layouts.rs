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
    /// The language's ISO 639 code in lower case, such as `fr`, which the keyboard's texts follow; empty when unknown.
    pub iso: String,
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
    /// Call it from one thread only, the face's thread that runs the relabel timer and the clicks, so its
    /// `LayoutChanged` events stay in order.
    fn refresh(&self) -> bool;

    /// Asks the app in front to switch `by` places along the list from the shown layout, as `step_to` counts.
    /// It only asks, and a step that lands on the shown layout (`by` 0, a lone layout) asks nothing and returns `Ok`;
    /// `current` changes when a later `refresh` sees the app switch, which may never happen.
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

/// The layout a step of `by` from `from` lands on in `list`, wrapping both ways; `None` for an empty list.
/// A `from` not in the list counts as the first layout; `Layouts::step` and the language key both count this way.
#[must_use]
pub fn step_to(list: &[LayoutId], from: Option<LayoutId>, by: i32) -> Option<LayoutId> {
    let len = i64::try_from(list.len()).ok().filter(|&n| n > 0)?;
    let at = from.and_then(|f| list.iter().position(|&l| l == f));
    let at = i64::try_from(at.unwrap_or_default()).ok()?;
    let to = usize::try_from((at + i64::from(by)).rem_euclid(len)).ok()?;
    list.get(to).copied()
}

/// The layouts service; it needs `InjectInput`, since a switch acts on the app in front.
pub struct LayoutsKey;

impl ServiceKey for LayoutsKey {
    type Api = dyn Layouts;
    const ID: ServiceId = ServiceId::new("layouts");
    const CAPABILITY: Option<Capability> = Some(Capability::InjectInput);
}

/// The keyboard now shows layout `id`.
/// The id is a hint: a listener reads `Layouts::current` for the table, which may have moved on since.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LayoutChanged {
    /// The layout shown now.
    pub id: LayoutId,
}

impl Event for LayoutChanged {
    const NAME: &'static str = "layout-changed";
}

#[cfg(test)]
mod tests;
