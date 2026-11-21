//! Voice typing's bus events (ADR-0042 decision 7): the key, its light, the panel's open, the bar's and panel's view,
//! their clicks and the text the keyboard types, as it stays the only typist. All need `Voice` but the sound level,
//! which holds no content.

mod act;
mod bar;
mod panel;

pub use act::{BarButton, CloudButton, FetchButton, PanelButton, VoiceClick};
pub use bar::{
    BarHint, BarNote, BarPlace, BarText, CloudWhy, Listening, LocalUse, Renewal, VoiceBar,
    VoiceLang, VoicePage, VoiceText,
};
pub use panel::{
    AddressFault, CardCheck, CardPart, CloudBand, CloudCard, CloudLine, CloudRow, CloudSheet,
    CompareCloud, CompareMic, CompareRow, CompareRun, CompareSheet, DeviceItem, DeviceLabel,
    DeviceName, DeviceRow, DeviceSheet, Ends, Fact, FactArg, FactValue, FailedCheck, FetchState,
    FolderLine, FolderList, HistoryRow, HistorySheet, KeyCheck, LinesPart, ModelRow, ModelSide,
    ModelState, ModelsSheet, OptionRow, OptionUnit, OptionValue, RunStep, ServiceMark, ServiceName,
    ServiceRow, ServiceSheet, Unavailable, UsePrice, VoiceHome, VoicePanel, VoiceSheet, VoiceTab,
};

use super::{PanelKey, PanelLine};
use crate::{Capability, Event, Secret};
use serde::Serialize;
use std::sync::Arc;

/// The gate of every event here but `VoiceLevel`.
const GATE: Option<Capability> = Some(Capability::Voice);

/// From the keyboard: the Dictation key was clicked, which starts or stops a dictation.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VoiceKey;

impl Event for VoiceKey {
    const NAME: &'static str = "voice-key";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the voice module: the Dictation key's light, so the keyboard draws it; told at each change and on `VoiceAsk`.
/// The face's state writes it by its lowercase name.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VoiceLit {
    /// Unlit: before the module speaks, and after a dictation stops, ends or fails.
    #[default]
    Off,
    /// Amber: a dictation started and the microphone is not open yet.
    Prep,
    /// Red: the microphone is open.
    Rec,
}

impl Event for VoiceLit {
    const NAME: &'static str = "voice-lit";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the ui: the tray's Dictation row or the bar's gear opens the Voice panel.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VoiceOpen;

impl Event for VoiceOpen {
    const NAME: &'static str = "voice-open";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From a late starter, such as the keyboard or the ui: the voice module tells `VoiceLit` and `VoiceView` again.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VoiceAsk;

impl Event for VoiceAsk {
    const NAME: &'static str = "voice-ask";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the voice module: the bar and the panel to draw, told once per change and shared, so clones are cheap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoiceView(pub Arc<VoicePage>);

impl VoiceView {
    /// Shares `page`.
    #[must_use]
    pub fn new(page: VoicePage) -> Self {
        Self(Arc::new(page))
    }
}

impl Event for VoiceView {
    const NAME: &'static str = "voice-view";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the ui: a click on the bar or the panel, with the round of the view it saw, so a click on an older view can be
/// dropped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoiceAct {
    /// The round of the view clicked.
    pub round: u32,
    /// What was clicked.
    pub click: VoiceClick,
}

impl Event for VoiceAct {
    const NAME: &'static str = "voice-act";
    const CAPABILITY: Option<Capability> = GATE;
}

/// To the keyboard: erase `erase` characters it typed, then type `text` paced by `gap_ms`, keeping every latch as it
/// was. The keyboard types nothing when `text` holds a hidden mark (`is_hidden`), so the module folds line breaks
/// first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextAsk {
    /// Names this request, so its answer names it too.
    pub round: u32,
    /// How many characters (Unicode scalars) to erase first, as Undo insert and Try again do.
    pub erase: u32,
    /// What to type then, in pieces under the keyboard's `text_cap`.
    pub text: Secret,
    /// The wait between typed pieces, in ms (`type_gap_ms`).
    pub gap_ms: u16,
}

impl Event for TextAsk {
    const NAME: &'static str = "text-ask";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the keyboard: what a `TextAsk` typed.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TextDone {
    /// The round of the request.
    pub round: u32,
    /// How many characters were typed, which Undo insert erases.
    pub count: u32,
    /// The erase and every piece went out.
    pub ok: bool,
}

impl Event for TextDone {
    const NAME: &'static str = "text-done";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the keyboard: a key clicked while the Voice panel's line is aimed at, which goes to that line instead of the
/// app (ADR-0049).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoiceLineKey(pub PanelKey);

impl Event for VoiceLineKey {
    const NAME: &'static str = "voice-line-key";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the voice module: the line the Voice panel's page shows, `None` while the panel is closed, told after the view
/// as it changes and on `VoiceAsk`. A key, address or model line is `PanelLine::Private`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoiceLine(pub Option<PanelLine>);

impl Event for VoiceLine {
    const NAME: &'static str = "voice-line";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the keyboard: the clipboard or Quick-fill panel opened, so the Voice panel closes as its Close does; it never
/// opens one, so a late one is harmless.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VoiceClose;

impl Event for VoiceClose {
    const NAME: &'static str = "voice-close";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the voice module: the sound level while listening, 0 to `LEVEL_FULL`, `level_hz` times a second. It holds no
/// content, so it needs no capability, and it never wakes the panel.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VoiceLevel {
    /// The level.
    pub level: u8,
}

impl Event for VoiceLevel {
    const NAME: &'static str = "voice-level";
}

#[cfg(test)]
mod tests;
