//! The bar's chip as QML reads it: what takes the voice, a cloud service or the model on this PC (owner, 2026-11-22),
//! by its kind, which picks its icon, its short name and the sentence readers hear.

use super::cloud::in_sentence;
use crate::I18n;
use crate::i18n::clean;
use crate::i18n::keys::vars::NAME;
use crate::i18n::keys::{VOICE_CUSTOM_SHORT, VOICE_ON_BAR_CLOUD, VOICE_ON_DEVICE};
use kx_module_api::ports::{BarMark, ServiceName};
use serde::Serialize;

/// A cloud service's kind, also its icon's name under the bar's icons.
const CLOUD: &str = "cloud";
/// The kind of a model on this PC, also its icon's name under the bar's icons.
const DEVICE: &str = "device";

/// The bar's chip: its kind, its name, whether that name is capped at the theme's width, and the line readers hear.
#[derive(Serialize)]
pub(super) struct Chip {
    kind: &'static str,
    name: String,
    /// True for a model's name, which may be capped at the theme's width; a service's short name shows whole.
    capped: bool,
    said: String,
}

/// The chip for `mark`, whose sentence names a service as the card and Compare do, so readers hear Custom one way
/// everywhere, and a model by the name the chip shows.
pub(super) fn of(t: &I18n, mark: &BarMark) -> Chip {
    let (kind, name, capped, said) = match mark {
        BarMark::Cloud(name) => {
            let said = t.voice_with(VOICE_ON_BAR_CLOUD, &[(NAME, in_sentence(t, name))]);
            (CLOUD, short(t, name), false, said)
        }
        BarMark::Device(name) => {
            let name = clean(name.expose());
            let said = t.voice_with(VOICE_ON_DEVICE, &[(NAME, name.clone())]);
            (DEVICE, name, true, said)
        }
    };
    Chip {
        kind,
        name,
        capped,
        said,
    }
}

/// Service `name` on the chip: a row's own name, or Custom's short one in the user's language.
fn short(t: &I18n, name: &ServiceName) -> String {
    match name {
        ServiceName::Named(n) => clean(n),
        ServiceName::Custom => t.text(VOICE_CUSTOM_SHORT),
    }
}
