//! Where the keyboard is (ADR-0041): the ui tells it at each change and on ask, so the chips follow the keyboard's
//! place. It carries no point or text, so it needs no capability.

use crate::Event;

/// Where the keyboard is.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum Board {
    /// On screen; a module takes this until it is told.
    #[default]
    Open,
    /// Minimised to its bubble.
    Minimised,
    /// Closed to the tray.
    Tray,
}

/// From the ui: the keyboard moved to `place`, or a module asked.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BoardPlace {
    /// Where the keyboard is now.
    pub place: Board,
}

impl Event for BoardPlace {
    const NAME: &'static str = "board-place";
}

/// To the ui: send `BoardPlace` again, for a module that starts later.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BoardPlaceAsk;

impl Event for BoardPlaceAsk {
    const NAME: &'static str = "board-place-ask";
}
