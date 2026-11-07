//! What a run of the panel's property test is made of: its steps and how often each comes.

use kx_module_api::Redacted;
use kx_module_api::ports::{ClipAct, Filter, PanelKey};
use proptest::prelude::*;

/// The highest row a click names, past any page.
const MAX_ROW: u8 = 8;
/// What the search line gets typed: letters, alef forms, a haraka, a blank, a hidden mark and an emoji.
const TYPED: [&str; 10] = [
    "e",
    "é",
    "ا",
    "أ",
    "\u{64e}",
    "o",
    " ",
    "x",
    "\u{200f}",
    "\u{1f600}",
];
/// One past the most a chip erases in a run.
const MAX_ERASE: u16 = 9;
/// One past the furthest a caret click lands, past any line.
const MAX_AT: u32 = 40;

/// One thing that happens while the panel is open.
#[derive(Debug, Clone)]
pub(super) enum Step {
    /// A key from our keyboard.
    Key(PanelKey),
    /// A click other than a tick.
    Act(ClipAct),
    /// A tick on row `row` of the last view, or of an older one when `stale`.
    Tick {
        /// It names an older view.
        stale: bool,
        /// The row clicked.
        row: u8,
    },
    /// A new copy of word `n`, wrapped.
    Copy(usize),
    /// The entry at this place, wrapped, leaves, as follow-the-clear or a sweep does.
    Remove(usize),
    /// The entry at this place, wrapped, is pinned or unpinned from elsewhere.
    Pin(usize),
}

impl Step {
    /// True when this step can only move the search's caret, while the line draws text.
    pub(super) fn is_caret(&self) -> bool {
        matches!(
            self,
            Self::Key(PanelKey::Left | PanelKey::Right) | Self::Act(ClipAct::Caret { .. })
        )
    }
}

/// Any key.
fn key() -> impl Strategy<Value = PanelKey> {
    let typed = prop::sample::select(TYPED.to_vec());
    prop_oneof![
        2 => Just(PanelKey::Up),
        3 => Just(PanelKey::Down),
        2 => Just(PanelKey::Left),
        2 => Just(PanelKey::Right),
        1 => Just(PanelKey::Enter),
        1 => Just(PanelKey::Esc),
        3 => Just(PanelKey::Tick),
        1 => Just(PanelKey::Erase),
        1 => Just(PanelKey::Delete),
        3 => typed.prop_map(|t| PanelKey::Text(Redacted::new(t.to_owned()))),
        2 => (0u16..MAX_ERASE, prop::sample::select(TYPED.to_vec())).prop_map(|(erase, t)| PanelKey::Word {
            erase,
            text: Redacted::new(t.to_owned()),
        }),
    ]
}

/// Any click but a tick.
fn act() -> impl Strategy<Value = ClipAct> {
    prop_oneof![
        prop::sample::select(Filter::ORDER.to_vec()).prop_map(ClipAct::Filter),
        any::<bool>().prop_map(|back| ClipAct::Page { back }),
        Just(ClipAct::Insert),
        Just(ClipAct::Pin),
        Just(ClipAct::Delete),
        Just(ClipAct::DeleteAll),
        any::<bool>().prop_map(ClipAct::Confirm),
        Just(ClipAct::Preview),
        Just(ClipAct::Back),
        Just(ClipAct::Close),
        (0..MAX_AT).prop_map(|at| ClipAct::Caret { at: at.into() }),
    ]
}

/// Any step.
pub(super) fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        6 => key().prop_map(Step::Key),
        4 => act().prop_map(Step::Act),
        3 => (any::<bool>(), 0..MAX_ROW).prop_map(|(stale, row)| Step::Tick { stale, row }),
        1 => any::<usize>().prop_map(Step::Copy),
        1 => any::<usize>().prop_map(Step::Remove),
        1 => any::<usize>().prop_map(Step::Pin),
    ]
}
