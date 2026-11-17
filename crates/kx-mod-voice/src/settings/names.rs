//! The `[voice]` values a word names: the bar's places and the ways a dictated text goes in, each name written once.

use kx_module_api::ports::BarPlace;

/// Each bar place's name in the setting.
pub const PLACES: [(&str, BarPlace); 3] = [
    ("above", BarPlace::Above),
    ("bottom", BarPlace::Bottom),
    ("top", BarPlace::Top),
];

/// How a dictated text goes into the box (owner, 2026-11-17).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Insert {
    /// The keyboard types it.
    Type,
    /// The clipboard history pastes it at once and puts the clipboard back; a password box is still typed.
    Paste,
}

/// Each way's name in the setting, typing first.
pub const INSERTS: [(&str, Insert); 2] = [("type", Insert::Type), ("paste", Insert::Paste)];

/// The bar place `name` names in the setting; `None` for any other text.
#[must_use]
pub fn place_named(name: &str) -> Option<BarPlace> {
    PLACES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, place)| *place)
}

/// The setting's name for `place`, which also names its text.
#[must_use]
pub fn place_name(place: BarPlace) -> &'static str {
    PLACES
        .iter()
        .find(|(_, p)| *p == place)
        .map_or(PLACES[0].0, |(name, _)| name)
}

/// The way `name` names in the setting; `None` for any other text.
#[must_use]
pub fn insert_named(name: &str) -> Option<Insert> {
    INSERTS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, way)| *way)
}
