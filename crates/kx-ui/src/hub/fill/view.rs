//! The Quick-fill panel as QML reads it (design "The panel"): every word in the keyboard's language, each user string
//! without hidden marks (ADR-0022) and with its own direction, and a hidden value only while the page carries it.

mod backup;
mod fields;
mod sheets;

use crate::I18n;
use crate::i18n::clean;
use crate::i18n::keys::{
    FILL_CHECKS, FILL_EMPTY, FILL_FILTER, FILL_MASK, FILL_NOTE, FILL_SEARCH, FILL_TITLE, FILL_WORD,
    WINDOW_FILL, field_button_id, fill_button_id, fill_filter_id, fill_note_id, place_kind_id,
};
use kx_module_api::ports::{
    FieldButton, FillButton, FillFilter, FillNote, FillPage, Label, PageNote, Spot,
};
use kx_module_api::{NoticeAsk, is_arabic, is_rtl};
use serde::Serialize;
use sheets::Shown;
use std::collections::BTreeMap;

/// The panel, as JSON.
#[derive(Serialize)]
pub(super) struct View {
    round: u32,
    title: String,
    words: Words,
    /// Each panel button's label by its id; the sheet says which show.
    buttons: BTreeMap<&'static str, String>,
    filters: Vec<Chip>,
    search: Search,
    page: Mark,
    highlight: Option<u8>,
    note: Option<String>,
    waiting: Option<Place>,
    empty: Option<String>,
    sheet: Shown,
}

/// The words outside the buttons: the window's name, the mask, a question's answers, the field lines' and the switch.
#[derive(Serialize)]
struct Words {
    window: String,
    mask: String,
    yes: String,
    no: String,
    value: String,
    label: String,
    hidden: String,
    checks: String,
}

/// A filter chip, by its id.
#[derive(Serialize)]
struct Chip {
    id: &'static str,
    label: String,
    on: bool,
}

/// The search line and its hint.
#[derive(Serialize)]
struct Search {
    text: String,
    /// The caret, in UTF-16 units of `text`.
    caret: u32,
    hint: String,
    rtl: bool,
    arabic: bool,
}

/// The page shown, `at` from 0, of `count`.
#[derive(Serialize)]
struct Mark {
    at: u16,
    count: u16,
}

/// A place's kind, for its icon, and its text.
#[derive(Serialize)]
struct Place {
    kind: &'static str,
    text: String,
}

/// A string to draw, without hidden marks, with the direction and font its own letters ask for.
#[derive(Serialize)]
struct Drawn {
    text: String,
    rtl: bool,
    arabic: bool,
}

/// A button, by its id, worded.
#[derive(Serialize)]
struct Button {
    id: &'static str,
    label: String,
}

/// `page` with every text in `t`.
pub(super) fn of(page: &FillPage, t: &I18n) -> View {
    View {
        round: page.round,
        title: t.text(FILL_TITLE),
        words: words(t),
        buttons: FillButton::ALL
            .into_iter()
            .map(|b| (fill_button_id(b), word(t, fill_button_id(b))))
            .collect(),
        filters: FillFilter::ALL
            .into_iter()
            .map(|f| Chip {
                id: fill_filter_id(f),
                label: t.text(&format!("{FILL_FILTER}{}", fill_filter_id(f))),
                on: f == page.filter,
            })
            .collect(),
        search: search(page, t),
        page: Mark {
            at: page.pages.at,
            count: page.pages.count,
        },
        highlight: page.highlight,
        note: page.note.map(|n| note(n, t)),
        waiting: page.waiting.as_ref().map(|s| Place {
            kind: place_kind_id(s.kind),
            text: t.fill_waiting(s.name.expose()),
        }),
        empty: page.empty.then(|| t.text(FILL_EMPTY)),
        sheet: sheets::of(page, t),
    }
}

/// The words outside the buttons.
fn words(t: &I18n) -> Words {
    Words {
        window: t.text(WINDOW_FILL),
        mask: t.text(FILL_MASK),
        yes: t.text(NoticeAsk::YES_NO.yes),
        no: t.text(NoticeAsk::YES_NO.no),
        value: word(t, field_button_id(FieldButton::Value)),
        label: word(t, field_button_id(FieldButton::Label)),
        hidden: word(t, field_button_id(FieldButton::Hidden)),
        checks: t.text(FILL_CHECKS),
    }
}

/// The search line of `page`, without hidden marks, and its hint.
fn search(page: &FillPage, t: &I18n) -> Search {
    let Drawn { text, rtl, arabic } = text(page.search.expose());
    Search {
        text,
        caret: *page.caret.expose(),
        hint: t.text(FILL_SEARCH),
        rtl,
        arabic,
    }
}

/// Note `n` worded, with the numbers it names.
fn note(n: PageNote, t: &I18n) -> String {
    match n.note {
        FillNote::Imported => t.fill_imported(n.counts),
        other => t.text(&format!("{FILL_NOTE}{}", fill_note_id(other))),
    }
}

/// The label of the button with id `id`, which is `fill.` then that id.
fn word(t: &I18n, id: &str) -> String {
    t.text(&format!("{FILL_WORD}{id}"))
}

/// The button `id`, labelled with the text `key` names.
fn button(t: &I18n, id: &'static str, key: &str) -> Button {
    Button {
        id,
        label: t.text(key),
    }
}

/// `raw` to draw: without hidden marks, with its own direction and font.
fn text(raw: &str) -> Drawn {
    let text = clean(raw);
    Drawn {
        rtl: is_rtl(&text),
        arabic: is_arabic(&text),
        text,
    }
}

/// `label` to draw: a table label's text in `t`, or the user's own.
fn label(label: &Label, t: &I18n) -> Drawn {
    match label {
        Label::Table(id) => text(&t.fill_label(id)),
        Label::Own(own) => text(own.expose()),
    }
}

/// Place `spot`'s kind id and its name to draw.
fn spot(spot: &Spot) -> (&'static str, Drawn) {
    (place_kind_id(spot.kind), text(spot.name.expose()))
}
