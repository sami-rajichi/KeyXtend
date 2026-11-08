//! Which keycaps show (P10 design "When the chips show"): by the box's kind on a selection, and Paste alone at a caret
//! in an editable box; each needs its own setting, and the helper's switch.

use kx_module_api::ports::{SelChip, SelKind};

/// The switches the keycaps follow, from `[selection]`.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each flag is one keycap's or the helper's own switch."
)]
pub struct Rules {
    /// The whole helper.
    pub on: bool,
    /// The Copy keycap.
    pub copy: bool,
    /// The Cut keycap.
    pub cut: bool,
    /// The Paste keycap.
    pub paste: bool,
    /// The Translate keycap.
    pub translate: bool,
    /// Paste by a caret just clicked into.
    pub caret_paste: bool,
}

/// What the box shows: a selection, or a bare caret.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum At {
    /// A selection that holds text.
    Selection,
    /// A caret with no selection; `clip` when the clipboard holds anything.
    Caret {
        /// The clipboard holds any format.
        clip: bool,
    },
}

/// The keycaps a box of `kind` shows `at` a selection or a caret, in `SelChip::ALL`'s order.
#[must_use]
pub fn chips(rules: &Rules, kind: SelKind, at: At) -> Vec<SelChip> {
    use SelChip::{Copy, Cut, Paste, Translate};
    let offered: &[SelChip] = match (kind, at) {
        _ if !rules.on => &[],
        (SelKind::Editable, At::Selection) => &SelChip::ALL,
        (SelKind::ReadOnly, At::Selection) => &[Copy, Translate],
        (SelKind::Terminal, At::Selection) => &[Copy, Paste, Translate],
        (SelKind::Editable, At::Caret { clip: true }) if rules.caret_paste => &[Paste],
        _ => &[],
    };
    let on = |chip: &&SelChip| match chip {
        Copy => rules.copy,
        Cut => rules.cut,
        Paste => rules.paste,
        Translate => rules.translate,
    };
    offered.iter().filter(on).copied().collect()
}

#[cfg(test)]
mod tests;
