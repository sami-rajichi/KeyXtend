//! The clipboard panel as QML reads it (spec §7): every text worded in the keyboard's language, sizes and times
//! formatted, user strings without hidden marks (ADR-0022), and pictures as URLs of the kept round, never pixels.

mod preview;

use super::{PART, PICTURES};
use crate::I18n;
use crate::i18n::clean;
use crate::i18n::keys::{
    CLIP_BACK, CLIP_CLOSE, CLIP_CONFIRM, CLIP_DELETE, CLIP_DELETE_ALL, CLIP_EMPTY, CLIP_EXPIRES,
    CLIP_FILTER, CLIP_FILTERS, CLIP_INSERT, CLIP_INSERT_ONE, CLIP_JOIN, CLIP_KIND, CLIP_NO,
    CLIP_NOTE, CLIP_OPEN_FOLDER, CLIP_PIN, CLIP_PINNED, CLIP_PREVIEW, CLIP_SEARCH,
    CLIP_SIZE_UNKNOWN, CLIP_TICKED, CLIP_TITLE, CLIP_UNPIN, CLIP_YES, WINDOW_CLIP, kind_id,
    note_id,
};
use kx_module_api::ports::{ClipKind, ClipPage, ClipRow};
use kx_module_api::{is_arabic, is_rtl};
use kx_theme::assets::IMAGE_SCHEME;
use preview::{Shown, preview};
use serde::Serialize;

/// The panel, as JSON.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct View {
    round: u32,
    title: String,
    kept: String,
    search: Search,
    filters: Vec<Chip>,
    rows: Vec<Row>,
    highlight: Option<u8>,
    page: Mark,
    ticked: u16,
    unpin: bool,
    /// Some entry is unpinned, so Delete all has something to delete.
    any_unpinned: bool,
    words: Words,
    preview: Option<Shown>,
    confirm: Option<Ask>,
    room: String,
    note: Option<String>,
    empty: Option<String>,
}

/// The search line and its hint.
#[derive(Serialize)]
struct Search {
    text: String,
    hint: String,
    arabic: bool,
    rtl: bool,
}

/// A filter chip, by its id in `CLIP_FILTERS`.
#[derive(Serialize)]
struct Chip {
    id: &'static str,
    label: String,
    on: bool,
}

/// One row; its id is a string, as JavaScript numbers lose a large one.
#[derive(Serialize)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each flag is one fact QML draws on its own: direction, font, pin and expiry."
)]
struct Row {
    id: String,
    kind: &'static str,
    title: String,
    rtl: bool,
    arabic: bool,
    /// The app, time and size in one line, as the row draws them.
    meta: String,
    tick: Option<u16>,
    pinned: bool,
    expires: bool,
    thumb: Option<String>,
    /// What readers hear: the title, its tick and its pin.
    name: String,
}

/// The page shown, `at` from 0, of `count`; the pager words its own number and buttons.
#[derive(Serialize)]
struct Mark {
    at: u16,
    count: u16,
}

/// The buttons' labels.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Words {
    window: String,
    close: String,
    preview: String,
    pin: String,
    delete: String,
    delete_all: String,
    insert: String,
    back: String,
    yes: String,
    no: String,
    open_folder: String,
    expires: String,
}

/// The Delete-all question, worded with how many entries it deletes.
#[derive(Serialize)]
struct Ask {
    text: String,
}

/// `page` with every text in `t`.
pub(super) fn of(page: &ClipPage, t: &I18n) -> View {
    View {
        round: page.round,
        title: t.text(CLIP_TITLE),
        kept: t.clip_kept(page.kept_days),
        search: search(page, t),
        filters: CLIP_FILTERS
            .iter()
            .map(|&(f, id)| Chip {
                id,
                label: t.text(&format!("{CLIP_FILTER}{id}")),
                on: f == page.filter,
            })
            .collect(),
        rows: page.rows.iter().map(|r| row(r, page.round, t)).collect(),
        highlight: page.highlight,
        page: Mark {
            at: page.pages.at,
            count: page.pages.count,
        },
        ticked: page.ticked,
        unpin: page.unpin,
        any_unpinned: page.unpinned > 0,
        words: words(page, t),
        preview: page.preview.as_ref().map(|p| preview(p, page.round, t)),
        confirm: page.confirm.map(|count| Ask {
            text: t.counted(CLIP_CONFIRM, usize::from(count)),
        }),
        room: t.clip_room(&page.room, page.cap),
        note: page
            .note
            .map(|n| t.text(&format!("{CLIP_NOTE}{}", note_id(n)))),
        empty: page.empty.then(|| t.text(CLIP_EMPTY)),
    }
}

/// The search line of `page`, without hidden marks, and its hint.
fn search(page: &ClipPage, t: &I18n) -> Search {
    let text = clean(page.search.expose());
    Search {
        arabic: is_arabic(&text),
        rtl: is_rtl(&text),
        text,
        hint: t.text(CLIP_SEARCH),
    }
}

/// Row `r` of view `round`; a row whose title is empty, or hidden marks alone, shows its kind's name.
fn row(r: &ClipRow, round: u32, t: &I18n) -> Row {
    let own = clean(r.title.expose());
    let (title, rtl, arabic) = if own.is_empty() {
        let name = kind_name(r.kind, t);
        (name.clone(), is_rtl(&name), is_arabic(&name))
    } else {
        (own, r.rtl, r.arabic)
    };
    let app = r.app.as_ref().map(|a| clean(a.expose()));
    let app = app.filter(|a| !a.is_empty());
    let size = r
        .bytes
        .map_or_else(|| t.text(CLIP_SIZE_UNKNOWN), |b| t.clip_size(b));
    let id = r.id.to_string();
    Row {
        kind: kind_id(r.kind),
        meta: t.clip_meta(app.as_deref(), &t.clip_when(&r.when), &size),
        name: reader_name(&title, r, t),
        thumb: r.thumb.as_ref().map(|_| url(round, &id)),
        id,
        title,
        rtl,
        arabic,
        tick: r.tick,
        pinned: r.pinned,
        expires: r.expires,
    }
}

/// What readers hear for row `r` shown as `title`: the title, then its tick and its pin, joined.
fn reader_name(title: &str, r: &ClipRow, t: &I18n) -> String {
    let mut parts = vec![title.to_owned()];
    parts.extend(r.tick.map(|n| t.counted(CLIP_TICKED, usize::from(n))));
    parts.extend(r.pinned.then(|| t.text(CLIP_PINNED)));
    parts.join(&t.text(CLIP_JOIN))
}

/// The buttons' labels for `page`: Pin or Unpin as it says, and Insert with the ticked count, or plain on the
/// preview page, where it acts on the previewed entry alone.
fn words(page: &ClipPage, t: &I18n) -> Words {
    Words {
        window: t.text(WINDOW_CLIP),
        close: t.text(CLIP_CLOSE),
        preview: t.text(CLIP_PREVIEW),
        pin: t.text(if page.unpin { CLIP_UNPIN } else { CLIP_PIN }),
        delete: t.text(CLIP_DELETE),
        delete_all: t.text(CLIP_DELETE_ALL),
        insert: if page.preview.is_some() {
            t.text(CLIP_INSERT_ONE)
        } else {
            t.counted(CLIP_INSERT, usize::from(page.ticked))
        },
        back: t.text(CLIP_BACK),
        yes: t.text(CLIP_YES),
        no: t.text(CLIP_NO),
        open_folder: t.text(CLIP_OPEN_FOLDER),
        expires: t.text(CLIP_EXPIRES),
    }
}

/// Kind `kind`'s name in `t`, as a row with no title shows it and readers hear an image preview.
fn kind_name(kind: ClipKind, t: &I18n) -> String {
    t.text(&format!("{CLIP_KIND}{}", kind_id(kind)))
}

/// The URL of picture `id` of view `round`.
fn url(round: u32, id: &str) -> String {
    format!("{IMAGE_SCHEME}://{PICTURES}/{round}{PART}{id}")
}
