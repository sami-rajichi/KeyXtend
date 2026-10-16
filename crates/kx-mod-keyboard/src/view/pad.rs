//! The scroll pad as the state draws it (spec §5.2): dimmed with a tip, doing nothing, while the scroll module is not
//! running, else showing the way and the speed it scrolls, as the bus last told.

use kx_module_api::ports::{Dir, ScrollChanged};
use serde::Serialize;

/// The pad's tip while the scroll module is not running: "The scroll pad is off".
pub const SCROLL_OFF_TIP: &str = "tip.scroll-off";

/// The pad's changing part.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct PadState {
    /// It does nothing now: the scroll module is not running.
    pub dim: bool,
    /// Its tip by translation key, while dimmed.
    pub tip: Option<&'static str>,
    /// The way it scrolls; none while still.
    pub dir: Option<Dir>,
    /// It scrolls at the fast speed.
    pub fast: bool,
}

/// The pad while the scroll module is in state `s`.
#[must_use]
pub fn pad_state(s: ScrollChanged) -> PadState {
    if !s.live {
        return PadState {
            dim: true,
            tip: Some(SCROLL_OFF_TIP),
            ..PadState::default()
        };
    }
    PadState {
        dir: s.dir,
        fast: s.fast,
        ..PadState::default()
    }
}
