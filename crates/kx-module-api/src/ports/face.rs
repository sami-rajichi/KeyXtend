//! The keyboard face: what the UI draws and the clicks it sends back.

use super::{InjectError, Rect, Screen, SwitchError};
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

/// A click the UI forwards to the keyboard, sent as JSON named by `act`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
        /// Place in the `presets` setting.
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
}

/// Why the keyboard could not do an action.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum FaceError {
    /// The board has no such key.
    #[error("the board has no such key")]
    UnknownKey,
    /// The index is outside its list.
    #[error("there is no choice at index {0}")]
    NoChoice(usize),
    /// The keys or text did not reach the app.
    #[error(transparent)]
    Inject(#[from] InjectError),
    /// The app in front could not be asked to switch layout.
    #[error(transparent)]
    Switch(#[from] SwitchError),
}

/// The keyboard as the UI sees it; `kx-mod-keyboard` provides it.
pub trait KeyboardFace: Send + Sync {
    /// The board to draw, as JSON: the size, and each key's box, names and whether it is dimmed.
    fn board_json(&self) -> String;

    /// The live state, as JSON: legends, lights, the language key, an open panel and the window's place.
    fn state_json(&self) -> String;

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
