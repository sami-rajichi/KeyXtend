//! What the strip hears and takes: the keyboard's place, its settings, its inputs, and a click on it.

use std::time::Duration;

use kx_module_api::ports::{SelChip, SelKind, SelSpot};

use crate::chips::Rules;

/// Where the keyboard is.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Board {
    /// On screen.
    #[default]
    Open,
    /// Minimised to its bubble.
    Minimised,
    /// Closed to the tray.
    Tray,
}

/// What the strip's rules take from `[selection]`.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct StripConfig {
    /// Which keycaps may show.
    pub rules: Rules,
    /// The strip shows while the keyboard is minimised.
    pub minimised: bool,
    /// How long the caret's Paste stays.
    pub caret: Duration,
}

/// One thing the strip hears.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Input {
    /// A look found `spot`; `click` when a press and release just came before it.
    Look {
        /// What the look found.
        spot: SelSpot,
        /// A press and release came just before the look.
        click: bool,
        /// The clipboard holds anything, from `SelRun::clip_holds`.
        clip: bool,
    },
    /// A look failed, or the box no longer holds its selection: what was seen is gone.
    Lost,
    /// A key the user sent to the app, other than Esc; a chip's own keys are not one, or an act's put-away would end.
    Key,
    /// A press in another app's window.
    Press,
    /// Esc, from our keyboard or the user's.
    Esc,
    /// One of our panels' lines is aimed at (true) or let go (false).
    Aimed(bool),
    /// The keyboard moved to this place.
    Board(Board),
    /// `chip` on the strip of `round` acted: its tick shows, then the strip ends.
    Acted {
        /// The round of the strip clicked.
        round: u32,
        /// The keycap that acted.
        chip: SelChip,
    },
    /// Time passed, for the caret's Paste to end.
    Tick,
}

/// A click on the live strip, with what its act needs.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Pick {
    /// The keycap clicked.
    pub chip: SelChip,
    /// The box the strip belongs to, for `SelRun::still`.
    pub box_id: u64,
    /// Its kind: a terminal takes the terminal keys.
    pub kind: SelKind,
}
