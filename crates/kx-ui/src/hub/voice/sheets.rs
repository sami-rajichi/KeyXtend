//! The Models page and the folder list as QML reads them: each model's name, size and marks, a download's progress
//! with its buttons, the models folder line, and the marks and page ids the other pages share.

use super::acts::FETCH;
use super::panel::{Cx, FOLDERS, Page, Sheet};
use super::parts::{Button, Hint, TOP_BEST, Tone, Top, word};
use crate::I18n;
use crate::i18n::clean;
use crate::i18n::keys::vars::{DONE, FREE, LANG, NEED, PATH, TOTAL};
use crate::i18n::keys::{
    VOICE_BEST, VOICE_DELETE_FILE, VOICE_DRIVES, VOICE_FOLDER, VOICE_FOLDER_OFFER, VOICE_FOLDERS,
    VOICE_GOT, VOICE_MARK, VOICE_PAUSED, VOICE_UNFIT, VOICE_UNTESTED, VoiceMark, fetch_button_id,
    mark_id,
};
use kx_module_api::ports::{
    FetchButton, FetchState, FolderLine, FolderList, ModelRow, ModelState, ModelsSheet,
    PanelButton, VoiceTab,
};
use serde::Serialize;

/// A model's row.
#[derive(Serialize)]
pub(super) struct ModelView {
    name: String,
    line: String,
    marks: Vec<Hint>,
    tick: bool,
    fetch: Option<Fetch>,
}

/// A download's progress, its share from 0 to 1, with Pause or Resume and Cancel.
#[derive(Serialize)]
pub(super) struct Fetch {
    share: f64,
    paused: bool,
    buttons: Vec<Button>,
}

/// The models folder line, with Change and Open.
#[derive(Serialize)]
pub(super) struct Room {
    text: String,
    buttons: Vec<Button>,
}

/// The Models page: the best model's note, the rows, the folder line, and Delete file, Download and Use, or the first
/// download's folder question.
pub(super) fn models(cx: &Cx<'_>, m: &ModelsSheet) -> Page {
    let t = cx.t;
    let lang = || (LANG, cx.lang());
    let rows = m.rows.iter().enumerate().map(|(at, r)| model(t, r, at));
    let room = m.folder.as_ref().map(|f| Room {
        text: folder(t, VOICE_FOLDER, f),
        buttons: vec![cx.button(PanelButton::Change), cx.button(PanelButton::Open)],
    });
    let sheet = Sheet::Models {
        rows: rows.collect(),
        untested: m.untested.then(|| t.voice_with(VOICE_UNTESTED, &[lang()])),
        room,
    };
    let mut page = Page::new(
        tab_id(VoiceTab::Models),
        cx.tab(VoiceTab::Models),
        cx.for_lang(),
        sheet,
    );
    // The model's name is drawn apart, bold as in the mock-up, after the line naming the language.
    page.top = m.best.as_ref().map(|b| {
        let text = t.voice_with(VOICE_BEST, &[lang()]);
        Top::new(TOP_BEST, text, Some(clean(b)))
    });
    if let Some(offer) = &m.offer {
        page.line = Some(folder(t, VOICE_FOLDER_OFFER, offer));
        page.foot = cx.ask(Tone::Primary);
    } else {
        let delete = cx.button_as(PanelButton::Delete, VOICE_DELETE_FILE);
        let used = cx.button(PanelButton::Use).tone(Tone::Primary);
        page.foot = vec![
            delete.tone(Tone::Danger),
            cx.button(PanelButton::Download),
            used,
        ];
    }
    page
}

/// Model row `r` at place `at`: its size or why it does not fit, its marks, or its download's progress. In use shows
/// only for a file that is here, so a missing default reads Default and Not downloaded.
fn model(t: &I18n, r: &ModelRow, at: usize) -> ModelView {
    let mut line = t.voice_size(r.bytes);
    let default = r.default.then_some(VoiceMark::Default);
    let marks = match r.state {
        _ if r.fetch.is_some() => vec![],
        ModelState::Ready | ModelState::Other if r.used => vec![Some(VoiceMark::Used), default],
        ModelState::Ready => vec![default, Some(VoiceMark::Ready)],
        ModelState::Missing => vec![default, Some(VoiceMark::Download)],
        ModelState::NotHere { need_mb, have_mb } => {
            // Until the port carries free memory, `$free` shows the total the module compared against.
            let args = [
                (NEED, t.voice_memory(need_mb)),
                (FREE, t.voice_memory(have_mb)),
            ];
            line = t.voice_with(VOICE_UNFIT, &args);
            vec![default, Some(VoiceMark::NotHere)]
        }
        ModelState::Other => vec![Some(VoiceMark::Other)],
    };
    let fetch = r.fetch.map(|f| {
        line = progress(t, f);
        fetch(t, f, at)
    });
    ModelView {
        name: clean(r.name.expose()),
        line,
        marks: marks.into_iter().flatten().map(|m| mark(t, m)).collect(),
        tick: r.tick,
        fetch,
    }
}

/// A download's buttons for the row at `at`.
fn fetch(t: &I18n, f: FetchState, at: usize) -> Fetch {
    let arg = i32::try_from(at).unwrap_or(i32::MAX);
    let first = if f.paused {
        FetchButton::Resume
    } else {
        FetchButton::Pause
    };
    let button = |b| {
        let id = fetch_button_id(b);
        Button::new(FETCH, id, arg, t.text(&word(id)))
    };
    Fetch {
        share: share(f.done, f.total),
        paused: f.paused,
        buttons: vec![button(first), button(FetchButton::Cancel)],
    }
}

/// A download's line: what it wrote of the whole, paused or not.
fn progress(t: &I18n, f: FetchState) -> String {
    let key = if f.paused { VOICE_PAUSED } else { VOICE_GOT };
    t.voice_with(
        key,
        &[(DONE, t.voice_size(f.done)), (TOTAL, t.voice_size(f.total))],
    )
}

/// Folder line `key` for `f`: its path and its drive's free room.
fn folder(t: &I18n, key: &str, f: &FolderLine) -> String {
    let args = [(PATH, clean(f.path.expose())), (FREE, t.voice_size(f.free))];
    t.voice_with(key, &args)
}

/// The folder list: the drives or a folder's subfolders, then Use this folder.
pub(super) fn folders(cx: &Cx<'_>, f: &FolderList) -> Page {
    let t = cx.t;
    let sub = f
        .folder
        .as_ref()
        .map_or_else(|| t.text(VOICE_DRIVES), |p| clean(p.expose()));
    let rows = f.rows.iter().map(|r| clean(r.expose())).collect();
    let mut page = Page::new(FOLDERS, t.text(VOICE_FOLDERS), sub, Sheet::Folders { rows });
    page.foot = vec![cx.button(PanelButton::Here).tone(Tone::Primary)];
    page
}

/// Mark `m` with its word.
pub(super) fn mark(t: &I18n, m: VoiceMark) -> Hint {
    Hint::new(mark_id(m), t.text(&mark_key(m)))
}

/// The key of mark `m`'s word.
pub(super) fn mark_key(m: VoiceMark) -> String {
    format!("{VOICE_MARK}{}", mark_id(m))
}

/// Tab `tab`'s page id.
pub(super) const fn tab_id(tab: VoiceTab) -> &'static str {
    crate::i18n::keys::voice_tab_id(tab)
}

/// `done` of `total`, from 0 to 1; 0 for no total.
fn share(done: u64, total: u64) -> f64 {
    #[allow(
        clippy::cast_precision_loss,
        reason = "a share needs no exact bytes, and model files are far below f64's exact range"
    )]
    let (done, total) = (done.min(total) as f64, total as f64);
    if total > 0.0 { done / total } else { 0.0 }
}
