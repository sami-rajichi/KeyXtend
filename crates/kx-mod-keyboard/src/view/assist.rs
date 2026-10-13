//! The mouse-assist side keys as the state draws them (spec §4.2): lit while their mode is on, and dimmed with a tip
//! while the mouse module is not running, as the bus last told.

use super::{DIM, KeyState, paint};
use crate::spec::{Assist, KeyKind};
use kx_module_api::ports::{AssistChanged, GrabState};

/// The tip of both keys while the mouse module is not running: "Mouse assist is off".
pub const MOUSE_OFF_TIP: &str = "tip.mouse-off";
/// Right-click's tip while it can be used: "Hold still to right-click; short clicks stay normal".
pub const RCLICK_TIP: &str = "tip.rclick";
/// Grab's tip while it can be used: "Press to pick up, move, then click to drop".
pub const GRAB_TIP: &str = "tip.grab";

/// The mouse module's state before the bus tells it: not running, every mode off.
pub const UNHEARD: AssistChanged = AssistChanged {
    live: false,
    rclick: false,
    grab: GrabState::Off,
};

/// Side key `a` of `kind` while the mouse module is in state `s`.
pub(super) fn assist_state(s: AssistChanged, a: Assist, kind: KeyKind) -> KeyState {
    if !s.live {
        return KeyState {
            paint: DIM,
            tip: Some(MOUSE_OFF_TIP),
            ..KeyState::default()
        };
    }
    let (on, tip) = match a {
        Assist::RClick => (s.rclick, RCLICK_TIP),
        Assist::Grab => (s.grab != GrabState::Off, GRAB_TIP),
    };
    KeyState {
        on,
        paint: paint(kind, on),
        tip: Some(tip),
        ..KeyState::default()
    }
}
