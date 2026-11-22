//! The bar's chip as QML reads it: what takes the voice, a cloud service or the model on this PC (owner, 2026-11-22),
//! by its kind, which picks its icon, its short name and the sentence readers hear with that name.

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

/// The chip for `mark`, whose sentence says the name the chip shows, so the eye and the ear agree.
pub(super) fn of(t: &I18n, mark: &BarMark) -> Chip {
    let (kind, name, capped, line) = match mark {
        BarMark::Cloud(name) => (CLOUD, short(t, name), false, VOICE_ON_BAR_CLOUD),
        BarMark::Device(name) => (DEVICE, clean(name.expose()), true, VOICE_ON_DEVICE),
    };
    Chip {
        kind,
        said: t.voice_with(line, &[(NAME, name.clone())]),
        capped,
        name,
    }
}

/// Service `name` on the chip: a row's own name, or Custom's short one in the user's language.
fn short(t: &I18n, name: &ServiceName) -> String {
    match name {
        ServiceName::Named(n) => clean(n),
        ServiceName::Custom => t.text(VOICE_CUSTOM_SHORT),
    }
}
