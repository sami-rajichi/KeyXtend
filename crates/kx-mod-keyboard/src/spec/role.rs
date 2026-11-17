//! What a click on a key does, as `board.toml` names it with `role`, and the mouse-assist modes a side key flips.

use super::LatchRole;
use kx_module_api::ports::Toggle;
use serde::{Deserialize, Serialize};

/// A mouse-assist mode a side key flips; the mouse module runs it (spec §5.1).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Assist {
    /// Right-click by holding still.
    RClick,
    /// Pick up at the next press, drop at the next click.
    Grab,
}

impl From<Assist> for Toggle {
    fn from(a: Assist) -> Self {
        match a {
            Assist::RClick => Self::RightClick,
            Assist::Grab => Self::Grab,
        }
    }
}

/// What a click on a key does.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Types its character in the layout shown; only the keys of a `chars` run have it.
    Char,
    /// Sends its scan code, with the latched modifiers.
    Send,
    /// Caps Lock: sends its scan code and lights while Caps Lock is on.
    Caps,
    /// Steps through or picks the layouts.
    Lang,
    /// Asks the mouse module to flip a mode; lit while it is on, dimmed while that module is not running.
    Assist(Assist),
    /// Comes in a later version: drawn dimmed and does nothing.
    Later,
    /// The Shortcuts key: a click shows or hides the shortcuts layer.
    Layer,
    /// The Power key: a click asks for the system's power dialog.
    Power,
    /// The clipboard key: a click asks the clipboard module to open or close its panel.
    Clip,
    /// The Quick-fill key: a click asks the vault module to open or close its panel.
    Fill,
    /// The Dictation key: a click asks the voice module to start or stop; it glows while the microphone is open.
    Voice,
    /// A shortcut on the layer; only the layer's generated cells have it.
    Shortcut,
    /// The layer's Next page key; only the layer's generated cell has it.
    Next,
    /// Latches a modifier; `board.toml` writes it as a table, `{ latch = "win", lone = true }`.
    #[serde(untagged)]
    Latch(LatchRole),
}

impl Role {
    /// True when a key with this role sends a scan code.
    #[must_use]
    pub fn sends(self) -> bool {
        matches!(self, Self::Char | Self::Send | Self::Caps | Self::Latch(_))
    }

    /// True for a role only a side key may have: one that asks another module, or shows the layer.
    #[must_use]
    pub fn side_only(self) -> bool {
        matches!(
            self,
            Self::Layer | Self::Power | Self::Clip | Self::Fill | Self::Voice
        )
    }
}
