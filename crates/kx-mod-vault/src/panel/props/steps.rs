//! The steps a property run takes: keys, clicks weighted toward the pages that show values, some from an older view,
//! and notebook changes from elsewhere.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    EntryKind, FieldButton, FillButton, FillClick, FillFilter, PanelKey, RowButton,
};
use proptest::prelude::*;

/// The highest row or field a click names, past any page.
const MAX_ROW: u8 = 7;
/// The most rows a page shows in a run; 0 counts as 1.
const MAX_ROWS: u8 = 6;
/// One past the furthest a caret click lands, past any line.
const MAX_AT: u32 = 40;
/// One past the most a chip erases in a run.
const MAX_ERASE: u16 = 9;
/// The share of clicks that name an older view.
const STALE: f64 = 0.25;
/// What gets typed: letters, Arabic with a haraka, a line break, a bidi override, a stripped mark, a word and an emoji.
const TYPED: [&str; 9] = [
    "a",
    "é",
    "ب",
    "\u{64e}",
    "\n",
    "\u{202e}",
    "\u{200f}",
    "Bank",
    "\u{1f600}",
];

/// One thing that happens while the panel is open.
#[derive(Debug, Clone)]
pub(super) enum Step {
    /// A key from our keyboard.
    Key(PanelKey),
    /// A click on the last view, or on the one before when `stale`.
    Click { stale: bool, click: FillClick },
    /// The entry at this place, wrapped, leaves, as Bring back or another computer's box may make it.
    Remove(usize),
    /// The entry at this place, wrapped, is filled, so the list moves.
    Use(usize),
    /// The entry at this place, wrapped, is replaced under its id, as Bring back may do.
    Replace(usize),
}

/// Any key.
fn key() -> impl Strategy<Value = PanelKey> {
    let typed = prop::sample::select(TYPED.to_vec());
    prop_oneof![
        2 => Just(PanelKey::Up),
        3 => Just(PanelKey::Down),
        1 => Just(PanelKey::Left),
        1 => Just(PanelKey::Right),
        2 => Just(PanelKey::Enter),
        1 => Just(PanelKey::Esc),
        1 => Just(PanelKey::Tick),
        1 => Just(PanelKey::Erase),
        1 => Just(PanelKey::Delete),
        3 => typed.prop_map(|t| PanelKey::Text(Redacted::new(t.to_owned()))),
        2 => (0u16..MAX_ERASE, prop::sample::select(TYPED.to_vec())).prop_map(|(erase, t)| PanelKey::Word {
            erase,
            text: Redacted::new(t.to_owned()),
        }),
    ]
}

/// The panel buttons that move between pages and change entries, which runs press most; Close comes rarer.
const PAGE_BUTTONS: [FillButton; 9] = [
    FillButton::New,
    FillButton::Edit,
    FillButton::Back,
    FillButton::Save,
    FillButton::Cancel,
    FillButton::Delete,
    FillButton::AddField,
    FillButton::OwnLabel,
    FillButton::Name,
];
/// The field buttons that show, hide, make or fill a value, which runs press most.
const VALUE_BUTTONS: [FieldButton; 5] = [
    FieldButton::Show,
    FieldButton::Hidden,
    FieldButton::Make,
    FieldButton::Value,
    FieldButton::Insert,
];
/// Fields a click names: the sample's three places and one past them.
const MAX_FIELD: u8 = 4;

/// Any panel button, the page ones most often.
fn button() -> impl Strategy<Value = FillButton> {
    prop_oneof![
        4 => prop::sample::select(PAGE_BUTTONS.to_vec()),
        1 => prop::sample::select(FillButton::ALL.to_vec()),
    ]
}

/// Any field button, the value ones most often.
fn field_button() -> impl Strategy<Value = FieldButton> {
    prop_oneof![
        5 => prop::sample::select(VALUE_BUTTONS.to_vec()),
        1 => prop::sample::select(FieldButton::ALL.to_vec()),
    ]
}

/// Any click.
fn click() -> impl Strategy<Value = FillClick> {
    prop_oneof![
        3 => (0..MAX_ROW, prop::sample::select(RowButton::ALL.to_vec()))
            .prop_map(|(row, button)| FillClick::Row { row, button }),
        5 => (0..MAX_FIELD, field_button())
            .prop_map(|(field, button)| FillClick::Field { field, button }),
        1 => prop::sample::select(FillFilter::ALL.to_vec()).prop_map(FillClick::Filter),
        2 => prop::sample::select(EntryKind::ALL.to_vec()).prop_map(FillClick::Kind),
        1 => any::<bool>().prop_map(|back| FillClick::Page { back }),
        6 => button().prop_map(FillClick::Button),
        1 => any::<bool>().prop_map(FillClick::Answer),
        1 => (0..=MAX_ROWS).prop_map(FillClick::Rows),
        3 => (0..MAX_AT).prop_map(|at| FillClick::Caret { at: at.into() }),
    ]
}

/// Any step; a quarter of the clicks name an older view.
pub(super) fn step() -> impl Strategy<Value = Step> {
    let stale = prop::bool::weighted(STALE);
    prop_oneof![
        4 => key().prop_map(Step::Key),
        10 => (stale, click()).prop_map(|(stale, click)| Step::Click { stale, click }),
        1 => any::<usize>().prop_map(Step::Remove),
        1 => any::<usize>().prop_map(Step::Use),
        1 => any::<usize>().prop_map(Step::Replace),
    ]
}
