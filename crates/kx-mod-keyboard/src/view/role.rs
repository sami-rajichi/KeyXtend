//! The keys whose role alone sets their state: a later key, a mouse mode, the Shortcuts and Power keys, the fill key,
//! dimmed with a tip while the vault module is not running (ADR-0037 decision 2), and the Dictation key's light.

use super::{DIM, KeyState, PREP, REC, StateInput, assist, layer, paint, power};
use crate::geom::KeyBox;
use crate::spec::{KeyKind, Role};
use kx_module_api::ports::{FillLive, VoiceLit};

/// The fill key's tip while the vault module is not running: "Quick-fill is off".
pub const FILL_OFF_TIP: &str = "tip.fill-off";
/// The Dictation key's name while the microphone starts, which readers hear in place of its own.
pub const PREP_NAME: &str = "voice.preparing";

/// Key `k`'s state when its role alone sets it: a later key, a mouse mode, the Shortcuts, Power, stopped fill or
/// Dictation key.
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
        Role::Voice => Some(voice_key(i.voice, k.kind)),
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

/// The Dictation key of `kind` under light `lit`: on and red while the microphone is open, amber with its preparing
/// name while it starts.
fn voice_key(lit: VoiceLit, kind: KeyKind) -> KeyState {
    let paint = match lit {
        VoiceLit::Off => paint(kind, false),
        VoiceLit::Prep => PREP,
        VoiceLit::Rec => REC,
    };
    KeyState {
        on: lit == VoiceLit::Rec,
        lit,
        spoken: (lit == VoiceLit::Prep).then_some(PREP_NAME),
        paint,
        ..KeyState::default()
    }
}
