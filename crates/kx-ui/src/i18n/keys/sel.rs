//! The selection strip's translation keys (P10): a keycap's word is `sel.` then its id, as in `sel.copy`, and its word
//! once it acted is `sel.done-` then its id. The match makes every new chip name its id.

use kx_module_api::ports::SelChip;

/// A keycap's word, which readers hear too: this, then its id.
pub const SEL_WORD: &str = "sel.";
/// A keycap's word once it acted, beside its tick: this, then its id.
pub const SEL_DONE: &str = "sel.done-";
/// The strip's name for readers.
pub const SEL_NAME: &str = "sel.name";

/// A keycap's id, which QML also picks its icon by and sends back on a click.
#[must_use]
pub const fn sel_chip_id(chip: SelChip) -> &'static str {
    match chip {
        SelChip::Copy => "copy",
        SelChip::Cut => "cut",
        SelChip::Paste => "paste",
        SelChip::Translate => "translate",
    }
}
