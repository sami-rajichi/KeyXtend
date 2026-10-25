//! The chip lane as the top bar reads it: each chip's text without hidden marks, its kind and direction, and the name
//! readers hear in the keyboard's language.

use crate::I18n;
use crate::i18n::clean;
use crate::i18n::keys::{CHIP_MORE, CHIP_UNDO};
use kx_module_api::ports::{Chip, ChipKind, Lane, ListPage, Suggested};
use serde::Serialize;

/// The lane before the first `Suggested`: prediction counts as on, with nothing to offer.
static START: Suggested = Suggested {
    round: 0,
    lane: Lane::Empty,
    forgetting: false,
    learn: true,
    on: true,
    list: None,
};

/// The lane, as JSON.
#[derive(Serialize)]
pub(super) struct View {
    round: u32,
    kind: Kind,
    chips: Vec<ChipView>,
    forgetting: bool,
    learn: bool,
    on: bool,
    list: Option<List>,
}

/// What the lane shows, as QML names it.
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum Kind {
    Chips,
    Locked,
    Off,
    Empty,
}

/// What a chip offers, as QML names it.
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum Offer {
    Word,
    Domain,
    Email,
    More,
    Undo,
}

impl From<ChipKind> for Offer {
    fn from(kind: ChipKind) -> Self {
        match kind {
            ChipKind::Word => Self::Word,
            ChipKind::Domain => Self::Domain,
            ChipKind::Email => Self::Email,
            ChipKind::More => Self::More,
            ChipKind::Undo => Self::Undo,
        }
    }
}

/// One chip as QML draws it, with the name readers hear.
#[derive(Serialize)]
struct ChipView {
    text: String,
    kind: Offer,
    rtl: bool,
    name: String,
}

/// The email list's page as QML draws it.
#[derive(Serialize)]
struct List {
    items: Vec<String>,
    page: u16,
    pages: u16,
}

/// `lane`, or the start's while none came, with the chips' names in `t`.
pub(super) fn of(lane: Option<&Suggested>, t: &I18n) -> View {
    let lane = lane.unwrap_or(&START);
    let (kind, chips) = match &lane.lane {
        Lane::Chips(chips) => {
            let named = chips.iter().map(|c| chip(c, t, lane.forgetting));
            (Kind::Chips, named.collect())
        }
        Lane::Locked => (Kind::Locked, Vec::new()),
        Lane::Off => (Kind::Off, Vec::new()),
        Lane::Empty => (Kind::Empty, Vec::new()),
    };
    View {
        round: lane.round,
        kind,
        chips,
        forgetting: lane.forgetting,
        learn: lane.learn,
        on: lane.on,
        list: lane.list.as_ref().map(page),
    }
}

/// Chip `c` as QML draws it: a word, domain or email is named by `chip.name`, or in forget mode by `chip.forget-name`;
/// More and Undo by their own text.
fn chip(c: &Chip, t: &I18n, forget: bool) -> ChipView {
    let text = clean(c.text.expose());
    let name = match c.kind {
        ChipKind::Word | ChipKind::Domain | ChipKind::Email => t.chip(&text, forget),
        ChipKind::More => t.text(CHIP_MORE),
        ChipKind::Undo => t.text(CHIP_UNDO),
    };
    ChipView {
        text,
        kind: c.kind.into(),
        rtl: c.rtl,
        name,
    }
}

/// The email list's page `list`, each row without hidden marks.
fn page(list: &ListPage) -> List {
    List {
        items: list.items.iter().map(|i| clean(i.expose())).collect(),
        page: list.page,
        pages: list.pages,
    }
}
