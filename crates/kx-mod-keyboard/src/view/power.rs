//! The Power key as the state draws it (spec §8.5): dimmed with a tip, doing nothing, while the power module is not
//! running, as the bus last told.

use super::{DIM, KeyState, paint};
use crate::spec::KeyKind;
use kx_module_api::ports::PowerChanged;

/// The Power key's tip while the power module is not running: "The power key is off".
pub const POWER_OFF_TIP: &str = "tip.power-off";

/// The Power key of `kind` while the power module is in state `s`.
pub(super) fn power_key(s: PowerChanged, kind: KeyKind) -> KeyState {
    if !s.live {
        return KeyState {
            dim: true,
            paint: DIM,
            tip: Some(POWER_OFF_TIP),
            ..KeyState::default()
        };
    }
    KeyState {
        paint: paint(kind, false),
        ..KeyState::default()
    }
}
