//! The bar's chip as QML reads it: what takes the voice, a cloud service or the model on this PC (owner, 2026-11-22),
//! by its kind, which picks its icon, its name and the sentence readers hear.

use super::cloud::{in_sentence, service};
use crate::I18n;
use crate::i18n::clean;
use crate::i18n::keys::vars::NAME;
use crate::i18n::keys::{VOICE_ON_BAR_CLOUD, VOICE_ON_DEVICE};
use kx_module_api::ports::BarMark;
use serde::Serialize;

/// A cloud service's kind, also its icon's name under the bar's icons.
const CLOUD: &str = "cloud";
/// The kind of a model on this PC, also its icon's name under the bar's icons.
const DEVICE: &str = "device";

/// The bar's chip: its kind, its name, and the line readers hear.
#[derive(Serialize)]
pub(super) struct Chip {
    kind: &'static str,
    name: String,
    said: String,
}

/// The chip for `mark`; a sentence names a service as a sentence does, and a model by its own name.
pub(super) fn of(t: &I18n, mark: &BarMark) -> Chip {
    match mark {
        BarMark::Cloud(name) => Chip {
            kind: CLOUD,
            name: service(t, name),
            said: t.voice_with(VOICE_ON_BAR_CLOUD, &[(NAME, in_sentence(t, name))]),
        },
        BarMark::Device(name) => {
            let name = clean(name.expose());
            Chip {
                kind: DEVICE,
                said: t.voice_with(VOICE_ON_DEVICE, &[(NAME, name.clone())]),
                name,
            }
        }
    }
}
