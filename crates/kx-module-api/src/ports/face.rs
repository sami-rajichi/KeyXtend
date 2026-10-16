//! The keyboard face: what the UI draws and the clicks it sends back.

use super::{Dir, Rect, Screen};
use crate::{Capability, REDACTED_MARKER, ServiceId, ServiceKey};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// A key's id on the board; its `Debug` hides it, since a run of taps would show what was typed.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KeyId(String);

impl KeyId {
    /// Wraps a board key id.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// The id as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for KeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED_MARKER)
    }
}

/// The most size presets a face offers: one per size button the UI names.
pub const MAX_PRESETS: usize = 3;

/// A click the UI forwards to the keyboard, sent as JSON named by `act`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "act", rename_all = "snake_case", deny_unknown_fields)]
pub enum FaceAction {
    /// A click on key `id`; `back` is the language key's left third.
    Tap {
        /// The key.
        id: KeyId,
        /// The click landed on the language key's back part.
        #[serde(default)]
        back: bool,
    },
    /// One size step smaller.
    Smaller,
    /// One size step bigger.
    Bigger,
    /// The size preset at `index`.
    Preset {
        /// Place in the `presets` setting, below `MAX_PRESETS`.
        index: usize,
    },
    /// A resize starts with the pointer at `x`, `y`: physical pixels, as `Screens::cursor` gives them.
    ResizeBegin {
        /// Pointer x; a fraction rounds.
        #[serde(deserialize_with = "super::geom::px")]
        x: i32,
        /// Pointer y; a fraction rounds.
        #[serde(deserialize_with = "super::geom::px")]
        y: i32,
    },
    /// The resize pointer moved to `x`, `y`, in the same pixels as `ResizeBegin`.
    ResizeAt {
        /// Pointer x; a fraction rounds.
        #[serde(deserialize_with = "super::geom::px")]
        x: i32,
        /// Pointer y; a fraction rounds.
        #[serde(deserialize_with = "super::geom::px")]
        y: i32,
    },
    /// The resize ends at the size it reached.
    ResizeEnd,
    /// The window ended at `rect`, and `screens` are the screens the UI can see.
    /// Rects are physical pixels in the space of `Screens::cursor`; fractions round.
    Placed {
        /// The window's place.
        rect: Rect,
        /// Each screen as the `Screens` port gives it: its whole `dpi` is sent as given, never `devicePixelRatio * 96`.
        screens: Vec<Screen>,
        /// How far the window reaches past the plate's right and bottom edges at size 1, in logical pixels: the half
        /// of the resize corner that hangs over, which placement keeps on the work area too.
        overhang: f64,
    },
    /// The UI's timer asks whether the app in front switched layout.
    Relabel,
    /// The language at `index` of the list was picked.
    LangPick {
        /// Place in the layouts list.
        index: usize,
    },
    /// Key `key` goes to the open panel, not to the app; the UI shows `rows` rows per page (`[ui] page_rows`).
    PanelKey {
        /// The key.
        key: KeyId,
        /// Rows per page, which turning a page moves by.
        rows: usize,
    },
    /// The pointer went down on key `id` (spec §4.6): it types now, and a repeating key again and again while held.
    Press {
        /// The key.
        id: KeyId,
    },
    /// The hold timer asks whether a held key's repeat is due.
    Tick,
    /// The pointer let go of key `id`, wherever it is: its repeat ends and nothing more types; another key's hold
    /// stays.
    LetGo {
        /// The key.
        id: KeyId,
    },
    /// Qt took the press away, or the keyboard hid: the repeat stops and nothing more types.
    Leave,
    /// A click on the scroll pad: an arrow, or Stop when `dir` is none.
    Pad {
        /// The arrow; none for the Stop button.
        dir: Option<Dir>,
    },
    /// The pointer went onto the scroll pad or off it.
    PadHover {
        /// The pointer is on the pad.
        on: bool,
    },
    /// Debug builds only: Caps Lock shows the other way round, with no key sent, for the memory churn.
    #[cfg(debug_assertions)]
    FlipCaps,
}

/// What the UI must do after an action.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct FaceReply {
    /// Redraw the board from `board_json`.
    pub board: bool,
    /// Redraw the state from `state_json`.
    pub state: bool,
    /// A notice to show, by translation key, such as a send or a language switch that failed.
    pub notice: Option<&'static str>,
    /// After a press or a tick, the keyboard still holds the key (spec §4.6), so the UI keeps its beat; false on
    /// every other reply.
    pub holding: bool,
}

/// Why the keyboard could not do an action; a send or switch that failed is a notice in the reply instead.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum FaceError {
    /// The board has no such key.
    #[error("the board has no such key")]
    UnknownKey,
    /// The index is outside its list.
    #[error("there is no choice at index {0}")]
    NoChoice(usize),
}

/// The keyboard as the UI sees it; `kx-mod-keyboard` provides it.
pub trait KeyboardFace: Send + Sync {
    /// The board to draw, as JSON: the size, and each key's box, names and whether it is dimmed.
    fn board_json(&self) -> String;

    /// The live state, as JSON: legends, lights, the language key, an open panel and the window's place.
    fn state_json(&self) -> String;

    /// The language of the layout shown, as `LangLabel::iso`; it only reads, so any thread may ask.
    fn lang(&self) -> String;

    /// The board's row height at size 1, in logical pixels, which the theme's drawn check measures caps by.
    fn row_px(&self) -> f64;

    /// Does `action` and says what to redraw; a send or switch that failed comes back as a notice in the reply.
    ///
    /// # Errors
    /// `UnknownKey` or `NoChoice` when the key or index is not on the board or in its list.
    fn act(&self, action: FaceAction) -> Result<FaceReply, FaceError>;
}

/// The keyboard face service; it needs no capability.
pub struct KeyboardFaceKey;

impl ServiceKey for KeyboardFaceKey {
    type Api = dyn KeyboardFace;
    const ID: ServiceId = ServiceId::new("keyboard-face");
    const CAPABILITY: Option<Capability> = None;
}

#[cfg(test)]
mod tests;
