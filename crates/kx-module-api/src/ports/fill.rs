//! Quick-fill's bus events (ADR-0038 decisions 3 and 4): whether the vault runs, the panel's state, its keys
//! and clicks, the label names, the fills and the emails need `Vault`; the open toggle, the close and the ask carry
//! nothing.

mod act;
mod backup;
mod field;
mod note;
mod view;

pub use act::{FieldButton, FillButton, FillClick, FillNames, RowButton};
pub use backup::{
    BackupAsk, BackupSheet, BoxState, FolderSheet, FoundRow, PasswordAsk, PasswordSheet, Pick,
    PlaceRow, Spot,
};
pub use field::{FieldLine, FieldValue, Label};
pub use note::{FillNote, NoteCounts, PageNote};
pub use view::{
    EditSheet, EntryKind, EntryRow, EntrySheet, FillFilter, FillPage, Hint, LabelCount, LabelValue,
    Sheet, Typing,
};

use super::{PanelKey, PanelLine};
use crate::{Capability, Event, Secret, is_hidden};
use std::sync::Arc;
use std::time::Duration;

/// The gate of every event here but `FillOpen`, `FillClose` and `FillAsk`, and of `FillNames`.
const GATE: Option<Capability> = Some(Capability::Vault);

/// From the keyboard or the ui: the side block's Quick-fill key, or the word row's Quick-fill chip, toggles the panel.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FillOpen;

impl Event for FillOpen {
    const NAME: &'static str = "fill-open";
}

/// From the keyboard: the clipboard panel opened, so an open panel closes as its close button does, and
/// `FillPanel { open: false }` follows; it never opens one, so a late one is harmless.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FillClose;

impl Event for FillClose {
    const NAME: &'static str = "fill-close";
}

/// From a late starter, such as the keyboard or word prediction: Quick-fill says again what it missed, and the vault
/// module answers with `FillLive`, `FillPanel` and `VaultEmails`. It carries nothing, so it needs no capability.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FillAsk;

impl Event for FillAsk {
    const NAME: &'static str = "fill-ask";
}

/// From the vault module: whether it runs, sent on its start, on `FillAsk` and on its stop; the keyboard dims its key.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FillLive {
    /// The vault module runs; false when it stops.
    pub live: bool,
}

impl FillLive {
    /// The vault module not running: before it speaks, and after it stops or fails.
    pub const OFF: Self = Self { live: false };
}

impl Event for FillLive {
    const NAME: &'static str = "fill-live";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the vault module: whether its panel is open, so the keyboard lights the key and sends panel keys.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FillPanel {
    /// The panel is open.
    pub open: bool,
}

impl Event for FillPanel {
    const NAME: &'static str = "fill-panel";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the keyboard: a key clicked while the panel is open, which goes to the panel instead of the app. Here
/// `PanelKey::Tick`, the Space key, types a space into the highlighted line or the search, as no row is ever ticked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FillKey(pub PanelKey);

impl Event for FillKey {
    const NAME: &'static str = "fill-key";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the vault module: the line its panel shows now, sent as it changes and on `FillAsk`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FillLine(pub PanelLine);

impl Event for FillLine {
    const NAME: &'static str = "fill-line";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the vault module: the panel to draw, shared, so every subscriber clones it cheaply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FillView(pub Arc<FillPage>);

impl FillView {
    /// Shares `page`.
    #[must_use]
    pub fn new(page: FillPage) -> Self {
        Self(Arc::new(page))
    }
}

impl Event for FillView {
    const NAME: &'static str = "fill-view";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the ui: a click on the panel with the round of the view it saw, so the module can drop a click on an older
/// view rather than fill what the user did not see.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FillAct {
    /// The round of the view clicked.
    pub round: u32,
    /// What was clicked.
    pub click: FillClick,
}

impl Event for FillAct {
    const NAME: &'static str = "fill-act";
    const CAPABILITY: Option<Capability> = GATE;
}

/// Text a fill types: it holds no control character or other hidden mark (`is_hidden`), so it never sends Enter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FillText(Secret);

impl FillText {
    /// `text` as fill text; `None` when it holds a line break, a Tab, another control or a hidden mark.
    #[must_use]
    pub fn new(text: Secret) -> Option<Self> {
        (!text.expose().chars().any(is_hidden)).then_some(Self(text))
    }

    /// The text.
    #[must_use]
    pub const fn text(&self) -> &Secret {
        &self.0
    }
}

/// One part of a fill.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FillPart {
    /// Text, typed by the route a letter takes: Unicode, or key presses in an app of `key_apps`.
    Text(FillText),
    /// One Tab key, as between a user and a password.
    Tab,
}

/// To the keyboard: let the latches go, then type `parts` at the caret, never Enter, and tell prediction one break and
/// never a value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FillType {
    /// Names this request, so its answer names it too.
    pub round: u32,
    /// What to type, in order.
    pub parts: Vec<FillPart>,
    /// The wait after each Tab before the next part, from the vault module's `tab_gap_ms`.
    pub tab_gap: Duration,
}

impl Event for FillType {
    const NAME: &'static str = "fill-type";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the keyboard: every part of the fill a `FillType` asked for went out, or not.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FillTyped {
    /// The round of the request.
    pub round: u32,
    /// Every part went out.
    pub ok: bool,
}

impl Event for FillTyped {
    const NAME: &'static str = "fill-typed";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the vault module: the notebook's emails, for prediction's `@@` list, which never learns them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VaultEmails {
    /// Every value saved under an Email label.
    pub emails: Vec<Secret>,
}

impl Event for VaultEmails {
    const NAME: &'static str = "vault-emails";
    const CAPABILITY: Option<Capability> = GATE;
}

#[cfg(test)]
mod tests;
