//! The chip lane as the top bar reads it: each chip's text without hidden marks, its kind, whether it is Arabic script
//! (`is_arabic`, as the keyboard's legends choose their font), and the name readers hear in the keyboard's language.

use crate::I18n;
use crate::i18n::clean;
use crate::i18n::keys::{CHIP_MORE, CHIP_UNDO};
use kx_module_api::is_arabic;
use kx_module_api::ports::{Chip, ChipKind, Lane, ListPage, Suggested};
use serde::Serialize;

/// The lane before the first `Suggested`: the module counts as stopped, so no menu shows until it tells a lane.
static START: Suggested = Suggested {
    round: 0,
    lane: Lane::Stopped,
    forgetting: false,
    learn: true,
    on: true,
    list: None,
};

/// The lane, as JSON.
#[derive(Serialize)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each flag is one fact about the lane that QML reads flat from its JSON."
)]
pub(super) struct View {
    round: u32,
    kind: Kind,
    chips: Vec<ChipView>,
    forgetting: bool,
    learn: bool,
    on: bool,
    list: Option<List>,
    /// The vault runs, so a password box shows its Quick-fill chip.
    fill: bool,
}

/// What the lane shows, as QML names it.
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum Kind {
    Chips,
    Locked,
    Off,
    Empty,
    Stopped,
}

/// What a chip offers, as QML names it.
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum Offer {
    Word,
    Domain,
    Email,
    Saved,
    More,
    Undo,
}

impl From<ChipKind> for Offer {
    fn from(kind: ChipKind) -> Self {
        match kind {
            ChipKind::Word => Self::Word,
            ChipKind::Domain => Self::Domain,
            ChipKind::Email => Self::Email,
            ChipKind::Saved => Self::Saved,
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
    arabic: bool,
    name: String,
}

/// One email list row as QML draws it: a known or saved email.
#[derive(Serialize)]
struct Row {
    text: String,
    arabic: bool,
    kind: Offer,
}

/// The email list's page as QML draws it.
#[derive(Serialize)]
struct List {
    items: Vec<Row>,
    page: u16,
    pages: u16,
}

/// `lane`, or the start's while none came, with the chips' names in `t` and whether the vault runs.
pub(super) fn of(lane: Option<&Suggested>, fill: bool, t: &I18n) -> View {
    let lane = lane.unwrap_or(&START);
    let (kind, chips) = match &lane.lane {
        Lane::Chips(chips) => {
            let named = chips.iter().map(|c| chip(c, t, lane.forgetting));
            (Kind::Chips, named.collect())
        }
        Lane::Locked => (Kind::Locked, Vec::new()),
        Lane::Off => (Kind::Off, Vec::new()),
        Lane::Empty => (Kind::Empty, Vec::new()),
        Lane::Stopped => (Kind::Stopped, Vec::new()),
    };
    View {
        round: lane.round,
        kind,
        chips,
        forgetting: lane.forgetting,
        learn: lane.learn,
        on: lane.on,
        list: lane.list.as_ref().map(page),
        fill,
    }
}

/// Chip `c` as QML draws it: a word, domain or email is named by `chip.name`, or in forget mode by `chip.forget-name`;
/// More and Undo by their own text.
fn chip(c: &Chip, t: &I18n, forget: bool) -> ChipView {
    let text = clean(c.text.expose());
    let name = match c.kind {
        ChipKind::Word | ChipKind::Domain | ChipKind::Email | ChipKind::Saved => {
            t.chip(&text, forget)
        }
        ChipKind::More => t.text(CHIP_MORE),
        ChipKind::Undo => t.text(CHIP_UNDO),
    };
    ChipView {
        arabic: is_arabic(&text),
        text,
        kind: c.kind.into(),
        name,
    }
}

/// The email list's page `list`, each row without hidden marks.
fn page(list: &ListPage) -> List {
    let row = |c: &Chip| {
        let text = clean(c.text.expose());
        Row {
            arabic: is_arabic(&text),
            text,
            kind: c.kind.into(),
        }
    };
    List {
        items: list.items.iter().map(row).collect(),
        page: list.page,
        pages: list.pages,
    }
}
