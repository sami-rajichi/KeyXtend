//! Snip and screen recording (ADR-0055, ADR-0056): ports that freeze the screens, record a part, ask where to save,
//! keep the Snips folder and play a video; the pages Snip's windows draw and their clicks; and bus events, gated by
//! the `Snip` event gate, as voice typing's are by `Voice` (ADR-0042), so the keyboard and the ui reach no port.

mod act;
mod cut;
mod editor;
mod file;
mod frame;
mod mic;
mod panel;
mod record;
mod shot;
mod speed;
mod store;
mod view;
mod waits;

pub use act::{Scale, SnipClick, SnipFit, TextSize, Thickness};
pub use editor::{
    Crop, CropFrame, CropKey, Drawing, EditKey, Editor, Ink, Mark, PictureEdit, Shape, TextEdit,
    Tool,
};
pub use file::{
    AskError, FileAsk, FileAskKey, FileKind, MediaOpen, MediaOpenKey, PlayError, SnipFile,
};
pub use frame::{Frame, Grip, Handle};
pub use mic::{MicPass, MicPassKey, MicPasses, MicSeal};
pub use panel::{CaptureSheet, FolderKey, SnipKeyboard, SnipSheet, SnipTab, VideoRow, VideoSheet};
pub use record::{
    Gear, RecordAsk, RecordDone, RecordEnd, RecordError, RecordFacts, RecordNote, RecordRun,
    Recorder, RecorderKey, Source,
};
pub use shot::{
    BGRX_BYTES, FreezeError, Frozen, FrozenScreen, Part, SHOT_CAP_MB, ScreenFreeze,
    ScreenFreezeKey, Shot, ShotCap,
};
pub use speed::{Speed, SpeedAsk};
pub use store::{SaveError, SaveTo, SnipFolder, SnipFolderKey, UnsavedFile};
pub use view::{
    AskKey, BarKey, CloseAsk, Countdown, Dim, Keep, Keycap, Picking, RecKey, Recording, SnipNote,
    SnipPage, SnipSpot, SnipStage, StripKey, Taken,
};
pub use waits::{PickLimits, RecordWaits};

use super::{PanelKey, PanelLine};
use crate::{Capability, Event};
use serde::Serialize;
use std::sync::Arc;

/// The gate of every snip event, which opens no port.
const GATE: Option<Capability> = Some(Capability::Snip);

/// From the keyboard: the Snip key was clicked, which starts a capture, or ends the one picking or recording.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SnipStart;

impl Event for SnipStart {
    const NAME: &'static str = "snip-start";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the snip module: what Snip's windows draw, told once per change and shared, so clones are cheap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnipView(pub Arc<SnipPage>);

impl SnipView {
    /// Shares `page`.
    #[must_use]
    pub fn new(page: SnipPage) -> Self {
        Self(Arc::new(page))
    }
}

impl Event for SnipView {
    const NAME: &'static str = "snip-view";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the ui: a click on Snip's windows, with the round of the view it saw, so a click on an older view can be
/// dropped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnipAct {
    /// The round of the view clicked.
    pub round: u32,
    /// What was clicked.
    pub click: SnipClick,
}

impl Event for SnipAct {
    const NAME: &'static str = "snip-act";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the snip module: the keyboard steps aside for a capture, or comes back once it ends or the editor opens. The ui
/// keeps it and its bubble out of every capture until `Back`, and answers a step aside with `SnipClick::Aside` once the
/// view told before it shows, which the freeze waits for until `[snip] aside_ms`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SnipAside {
    /// It turns into its bubble.
    Bubble,
    /// It and its bubble hide.
    Hidden,
    /// It stays as `keyboard = shown` asks, for a snip started from the editor or its question; no `Back` follows.
    Stay,
    /// It comes back as it was.
    Back,
}

impl Event for SnipAside {
    const NAME: &'static str = "snip-aside";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the snip module: the Snip key's light, lit from a capture's start until it ends or its editor opens. It is told
/// at each change and on `SnipAsk`.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SnipLit {
    /// Unlit: before the module speaks, and once a capture ends.
    #[default]
    Off,
    /// Lit: a capture runs.
    On,
}

impl Event for SnipLit {
    const NAME: &'static str = "snip-lit";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the snip module, at each change and on `SnipAsk`: the line Snip's windows show, `None` while none shows.
/// The editor's text line is `PanelLine::Private`; any other is `PanelLine::None`, which still takes Enter and Esc
/// (ADR-0057).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnipLine(pub Option<PanelLine>);

impl Event for SnipLine {
    const NAME: &'static str = "snip-line";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From the keyboard: a key clicked while a Snip window is aimed at, which goes to it instead of the app: letters to
/// the editor's text line, Esc to end a capture or answer Back (ADR-0040).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnipLineKey(pub PanelKey);

impl Event for SnipLineKey {
    const NAME: &'static str = "snip-line-key";
    const CAPABILITY: Option<Capability> = GATE;
}

/// From a late starter, such as the keyboard: the snip module tells `SnipLit` and `SnipLine` again, even unchanged.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SnipAsk;

impl Event for SnipAsk {
    const NAME: &'static str = "snip-ask";
    const CAPABILITY: Option<Capability> = GATE;
}

#[cfg(test)]
mod tests;
