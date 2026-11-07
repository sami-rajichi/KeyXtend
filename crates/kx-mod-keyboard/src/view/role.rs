//! The keys whose role alone sets their state: a later key, a mouse mode, the Shortcuts and Power keys, and the fill
//! key, dimmed with a tip while the vault module is not running (ADR-0037 decision 2).

use super::{DIM, KeyState, StateInput, assist, layer, power};
use crate::geom::KeyBox;
use crate::spec::Role;
use kx_module_api::ports::FillLive;

/// The fill key's tip while the vault module is not running: "Quick-fill is off".
pub const FILL_OFF_TIP: &str = "tip.fill-off";

/// Key `k`'s state when its role alone sets it: a later key, a mouse mode, the Shortcuts, Power or stopped fill key.
pub(super) fn role_state(i: &StateInput<'_>, k: &KeyBox) -> Option<KeyState> {
    match k.role {
        Role::Later => Some(KeyState {
            dim: true,
            paint: DIM,
            ..KeyState::default()
        }),
        Role::Assist(a) => Some(assist::assist_state(i.assist, a, k.kind)),
        Role::Layer => Some(layer::layer_key(i.layer, k.kind)),
        Role::Power => Some(power::power_key(i.power, k.kind)),
        Role::Fill => fill_key(i.fill),
        _ => None,
    }
}

/// The fill key while the vault module is in state `s`; `None` while it runs, as its open panel then lights it.
fn fill_key(s: FillLive) -> Option<KeyState> {
    (!s.live).then(|| KeyState {
        dim: true,
        paint: DIM,
        tip: Some(FILL_OFF_TIP),
        ..KeyState::default()
    })
}
