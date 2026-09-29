//! The keyboard face: what the UI draws and the clicks it sends back.

use super::{InjectError, Rect, SwitchError};
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
    /// The window ended at `rect`, and `screens` are the work areas the UI can see.
    /// Both are physical pixels in the space of `Screens::cursor`; fractions round.
    Placed {
        /// The window's place.
        rect: Rect,
        /// The work area of each screen.
        screens: Vec<Rect>,
    },
    /// The UI's timer asks whether the app in front switched layout.
    Relabel,
    /// The language at `index` of the list was picked.
    LangPick {
        /// Place in the layouts list.
        index: usize,
    },
    /// Key `key` goes to the open panel, not to the app.
    PanelKey {
        /// The key.
        key: KeyId,
    },
}

/// What the UI must do after an action.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct FaceReply {
    /// Redraw the board from `board_json`.
    pub board: bool,
    /// Redraw the state from `state_json`.
    pub state: bool,
    /// A notice to show, by translation key.
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
    /// The board to draw, as JSON: keys, places and legends.
    fn board_json(&self) -> String;

    /// The live state, as JSON: latches, Caps, size and language.
    fn state_json(&self) -> String;

    /// Does `action` and says what to redraw.
    ///
    /// # Errors
    /// A `FaceError` when the key or index is unknown, or the input or switch failed.
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
mod tests {
    use super::*;

    const KEY: &str = "key-q";
    /// The number of `FaceAction` variants; `row` fails to build when one is added.
    const VARIANTS: usize = 11;

    /// Each variant's place in `wire`, by an exhaustive match.
    fn row(action: &FaceAction) -> usize {
        match action {
            FaceAction::Tap { .. } => 0,
            FaceAction::Smaller => 1,
            FaceAction::Bigger => 2,
            FaceAction::Preset { .. } => 3,
            FaceAction::ResizeBegin { .. } => 4,
            FaceAction::ResizeAt { .. } => 5,
            FaceAction::ResizeEnd => 6,
            FaceAction::Placed { .. } => 7,
            FaceAction::Relabel => 8,
            FaceAction::LangPick { .. } => 9,
            FaceAction::PanelKey { .. } => 10,
        }
    }

    /// The tap, placement and panel-key samples, which need built values.
    fn built() -> [FaceAction; 3] {
        let rect = Rect {
            x: -8,
            y: 40,
            w: 900,
            h: 300,
        };
        let tap = FaceAction::Tap {
            id: KeyId::new(KEY),
            back: true,
        };
        let placed = FaceAction::Placed {
            rect,
            screens: vec![rect],
        };
        let panel = FaceAction::PanelKey {
            key: KeyId::new(KEY),
        };
        [tap, placed, panel]
    }

    /// Every action and the exact JSON the UI sends for it.
    fn wire() -> Vec<(FaceAction, &'static str)> {
        let [tap, placed, panel] = built();
        vec![
            (tap, r#"{"act":"tap","id":"key-q","back":true}"#),
            (FaceAction::Smaller, r#"{"act":"smaller"}"#),
            (FaceAction::Bigger, r#"{"act":"bigger"}"#),
            (
                FaceAction::Preset { index: 2 },
                r#"{"act":"preset","index":2}"#,
            ),
            (
                FaceAction::ResizeBegin { x: 10, y: -20 },
                r#"{"act":"resize_begin","x":10,"y":-20}"#,
            ),
            (
                FaceAction::ResizeAt { x: 30, y: 40 },
                r#"{"act":"resize_at","x":30,"y":40}"#,
            ),
            (FaceAction::ResizeEnd, r#"{"act":"resize_end"}"#),
            (
                placed,
                r#"{"act":"placed","rect":{"x":-8,"y":40,"w":900,"h":300},"screens":[{"x":-8,"y":40,"w":900,"h":300}]}"#,
            ),
            (FaceAction::Relabel, r#"{"act":"relabel"}"#),
            (
                FaceAction::LangPick { index: 1 },
                r#"{"act":"lang_pick","index":1}"#,
            ),
            (panel, r#"{"act":"panel_key","key":"key-q"}"#),
        ]
    }

    #[test]
    fn every_action_has_one_exact_json_form() {
        let table = wire();
        let rows: Vec<usize> = table.iter().map(|(a, _)| row(a)).collect();
        assert_eq!(
            rows,
            (0..VARIANTS).collect::<Vec<_>>(),
            "one row per variant, in order"
        );
        for (action, json) in table {
            assert_eq!(serde_json::to_string(&action).unwrap(), json);
            assert_eq!(serde_json::from_str::<FaceAction>(json).unwrap(), action);
        }
    }

    #[test]
    fn pointer_and_window_places_may_have_fractions() {
        let at = r#"{"act":"resize_at","x":10.5,"y":-0.4}"#;
        let want = FaceAction::ResizeAt { x: 11, y: 0 };
        assert_eq!(serde_json::from_str::<FaceAction>(at).unwrap(), want);
        let placed = r#"{"act":"placed","rect":{"x":0.5,"y":1,"w":2,"h":3},"screens":[]}"#;
        let FaceAction::Placed { rect, .. } = serde_json::from_str(placed).unwrap() else {
            panic!("not a placement");
        };
        assert_eq!(rect.x, 1);
    }

    #[test]
    fn the_ui_names_the_action_in_act() {
        let tap: FaceAction = serde_json::from_str(r#"{"act":"tap","id":"key-q"}"#).unwrap();
        let want = FaceAction::Tap {
            id: KeyId::new(KEY),
            back: false,
        };
        assert_eq!(tap, want);
        let pick: FaceAction = serde_json::from_str(r#"{"act":"lang_pick","index":3}"#).unwrap();
        assert_eq!(pick, FaceAction::LangPick { index: 3 });
        assert!(serde_json::from_str::<FaceAction>(r#"{"act":"jump"}"#).is_err());
        let extra = r#"{"act":"preset","index":1,"size":2}"#;
        assert!(serde_json::from_str::<FaceAction>(extra).is_err());
    }

    #[test]
    fn debug_never_shows_which_key_was_pressed() {
        let actions: Vec<FaceAction> = wire().into_iter().map(|(a, _)| a).collect();
        let shown = format!("{actions:?}");
        assert!(!shown.contains(KEY), "{shown}");
        assert!(shown.contains(REDACTED_MARKER));
        assert_eq!(KeyId::new(KEY).as_str(), KEY);
    }

    #[test]
    fn a_reply_redraws_nothing_by_default() {
        let reply = FaceReply::default();
        assert!(!reply.board && !reply.state && reply.notice.is_none());
    }

    #[test]
    fn the_face_needs_no_capability() {
        assert_eq!(KeyboardFaceKey::ID.as_str(), "keyboard-face");
        assert_eq!(KeyboardFaceKey::CAPABILITY, None);
    }
}
