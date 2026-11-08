//! The strip as QML reads it: each keycap's id, word and done word in the keyboard's language, the tick, whether icons
//! and words show, the strip's name for readers, and the scale of the selection's screen. It holds no box.

use super::Kept;
use crate::I18n;
use crate::i18n::keys::{SEL_DONE, SEL_NAME, SEL_WORD, sel_chip_id};
use kx_module_api::ports::{BASE_DPI, SelChip};
use serde::Serialize;

/// The strip, as JSON.
#[derive(Serialize)]
pub(super) struct View {
    round: u32,
    chips: Vec<Cap>,
    /// The id of the keycap that acted, which shows its tick and done word.
    acted: Option<&'static str>,
    icons: bool,
    words: bool,
    /// The strip's name for readers.
    name: String,
    /// The selection screen's scale, so QML measures the strip in that screen's physical pixels.
    scale: f64,
}

/// One keycap: its id, its word, which readers hear too, and its word once it acted.
#[derive(Serialize)]
struct Cap {
    id: &'static str,
    word: String,
    done: String,
}

/// Strip `k` with its words in `t`.
pub(super) fn of(k: &Kept, t: &I18n) -> View {
    let cap = |c: SelChip| {
        let id = sel_chip_id(c);
        Cap {
            id,
            word: t.text(&format!("{SEL_WORD}{id}")),
            done: t.text(&format!("{SEL_DONE}{id}")),
        }
    };
    View {
        round: k.round,
        chips: k.chips.iter().copied().map(cap).collect(),
        acted: k.acted.map(sel_chip_id),
        icons: k.legend.icons(),
        words: k.legend.words(),
        name: t.text(SEL_NAME),
        scale: f64::from(k.spot.screen.dpi) / f64::from(BASE_DPI),
    }
}
