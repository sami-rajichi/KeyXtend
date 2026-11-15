//! Compare models and History as QML reads them: the set text and its models, each model's turn, the results' table,
//! and each dictated text with its line and the name readers hear, then each page's footer.

use super::acts::COMPARE;
use super::panel::{Cx, Page, Sheet, count};
use super::parts::{Button, Hint, TOP_KEPT, TOP_STEP, Tone, Top, word};
use super::sheets::{mark, mark_key, tab_id};
use crate::I18n;
use crate::i18n::clean;
use crate::i18n::keys::vars::{AT, DATE, DEVICE, LANG, MODEL, OF, TEXTS, WAIT, WHEN, WORDS, WRONG};
use crate::i18n::keys::{
    LANG_NAME, VOICE_COL, VOICE_COLS, VOICE_CONFIRM, VOICE_EMPTY, VOICE_HISTORY_SUB, VOICE_INSERT,
    VOICE_INSERT_ONE, VOICE_INTRO, VOICE_INTRO_FREE, VOICE_JOIN, VOICE_KEPT, VOICE_META,
    VOICE_PINNED, VOICE_READY, VOICE_RECORDING, VOICE_RESULT_LINE, VOICE_START, VOICE_STEP,
    VOICE_TEXTS, VOICE_TICKED, VOICE_UNPIN, VOICE_WRONG_OF, VoiceMark, mark_id, panel_button_id,
};
use kx_module_api::ports::{
    CompareRow, CompareRun, CompareSheet, DeviceLabel, HistoryRow, HistorySheet, PanelButton,
    RunStep, VoiceTab,
};
use serde::Serialize;

/// A model's row while Compare runs.
#[derive(Serialize)]
pub(super) struct RunView {
    name: String,
    mark: Hint,
}

/// A model's row in the results' table; `wrong` is `None`, and `text` shows, for a language with no set text.
#[derive(Serialize)]
pub(super) struct ResultView {
    name: String,
    wrong: Option<String>,
    wait: String,
    memory: String,
    text: Option<String>,
    tick: bool,
}

/// A dictated text, with its line and the name readers hear.
#[derive(Serialize)]
pub(super) struct HistoryView {
    id: u64,
    text: String,
    rtl: bool,
    arabic: bool,
    meta: String,
    pinned: bool,
    tick: Option<u16>,
    name: String,
}

/// Compare models: the set text with Start, each model's turn with Cancel, or the results with Compare again and Use.
pub(super) fn compare(cx: &Cx<'_>, c: &CompareSheet) -> Page {
    let t = cx.t;
    let new = |sub, sheet| compare_page(cx, sub, sheet);
    match c {
        CompareSheet::Ready {
            models,
            device,
            recording,
        } => ready(cx, *models, device, *recording),
        CompareSheet::Running { at, of, name, rows } => {
            let sheet = Sheet::Running {
                rows: rows.iter().map(|r| run(t, r)).collect(),
            };
            let mut page = new(cx.for_lang(), sheet);
            let step = t.voice_with(VOICE_STEP, &[(AT, at.to_string()), (OF, of.to_string())]);
            page.top = Some(Top::new(TOP_STEP, step, Some(clean(name.expose()))));
            page.foot = vec![cx.button(PanelButton::Cancel)];
            page
        }
        CompareSheet::Results { day, device, rows } => {
            let args = [(DATE, t.voice_date(*day)), (DEVICE, cx.device(device))];
            let head = VOICE_COLS.map(|c| t.text(&format!("{VOICE_COL}{c}")));
            let rows = rows.iter().map(|r| result(t, r)).collect();
            let sheet = Sheet::Results {
                head: head.into(),
                rows,
            };
            let mut page = new(t.voice_with(VOICE_RESULT_LINE, &args), sheet);
            page.top = Some(Top::new(TOP_KEPT, t.text(VOICE_KEPT), None));
            let used = cx.button(PanelButton::Use).tone(Tone::Primary);
            page.foot = vec![cx.button(PanelButton::Rerun), used];
            page
        }
    }
}

/// Compare's page showing `sheet` under line `sub`.
fn compare_page(cx: &Cx<'_>, sub: String, sheet: Sheet) -> Page {
    let tab = VoiceTab::Compare;
    Page::new(tab_id(tab), cx.tab(tab), sub, sheet)
}

/// The set text with the `models` it runs on `device`, and Start, or Cancel while the user reads it.
fn ready(cx: &Cx<'_>, models: u16, device: &DeviceLabel, recording: bool) -> Page {
    let t = cx.t;
    let line = t.voice_counted(
        VOICE_READY,
        usize::from(models),
        &[(DEVICE, cx.device(device))],
    );
    let intro = if cx.read.is_some() {
        VOICE_INTRO
    } else {
        VOICE_INTRO_FREE
    };
    let sheet = Sheet::Ready {
        intro: t.text(intro),
        text: cx.read.map(str::to_owned),
        line,
        recording: recording.then(|| t.text(VOICE_RECORDING)),
    };
    let mut page = compare_page(cx, cx.for_lang(), sheet);
    let start = Button::new(COMPARE, "", 0, t.text(VOICE_START)).on(models > 0);
    page.foot = if recording {
        vec![cx.button(PanelButton::Cancel)]
    } else {
        vec![start.tone(Tone::Primary)]
    };
    page
}

/// A model's turn while Compare runs.
fn run(t: &I18n, r: &CompareRun) -> RunView {
    let mark = match r.step {
        RunStep::Done { wait_ms } => {
            let key = mark_key(VoiceMark::Done);
            let text = t.voice_with(&key, &[(WAIT, t.voice_secs(wait_ms))]);
            Hint::new(mark_id(VoiceMark::Done), text)
        }
        RunStep::Failed => mark(t, VoiceMark::RunFailed),
        RunStep::Writing => mark(t, VoiceMark::Writing),
        RunStep::Waiting => mark(t, VoiceMark::Waiting),
    };
    RunView {
        name: clean(r.name.expose()),
        mark,
    }
}

/// A model's numbers.
fn result(t: &I18n, r: &CompareRow) -> ResultView {
    let wrong = r.wrong.map(|w| {
        let args = [(WRONG, w.to_string()), (WORDS, r.words.to_string())];
        t.voice_with(VOICE_WRONG_OF, &args)
    });
    ResultView {
        name: clean(r.name.expose()),
        wrong,
        wait: t.voice_secs(r.wait_ms),
        memory: t.voice_memory(r.mb),
        text: r.text.as_ref().map(|x| clean(x.expose())),
        tick: r.tick,
    }
}

/// History: Delete all in the header, the texts, then Copy, Pin, Delete and Insert, or Delete all's question.
pub(super) fn history(cx: &Cx<'_>, h: &HistorySheet) -> Page {
    let t = cx.t;
    let texts = t.counted(VOICE_TEXTS, count(h.total));
    let sheet = Sheet::History {
        rows: h.rows.iter().map(|r| history_row(t, r)).collect(),
        empty: (h.total == 0).then(|| t.text(VOICE_EMPTY)),
        unpin: h.unpin,
    };
    let tab = VoiceTab::History;
    let sub = t.voice_with(VOICE_HISTORY_SUB, &[(TEXTS, texts)]);
    let mut page = Page::new(tab_id(tab), cx.tab(tab), sub, sheet);
    let all = cx.button(PanelButton::DeleteAll);
    let lists = cx.panel.acts.contains(&PanelButton::DeleteAll);
    page.extra = Some(all.on(lists && !h.confirm));
    if h.confirm {
        page.line = Some(t.counted(VOICE_CONFIRM, count(h.unpinned)));
        page.foot = cx.ask(Tone::Danger);
        return page;
    }
    let pin = if h.unpin {
        VOICE_UNPIN.to_owned()
    } else {
        word(panel_button_id(PanelButton::Pin))
    };
    let ticked = usize::from(cx.panel.ticked);
    let insert = if ticked > 1 {
        t.counted(VOICE_INSERT, ticked)
    } else {
        t.text(VOICE_INSERT_ONE)
    };
    let insert = Button::panel_as(PanelButton::Insert, insert, &cx.panel.acts);
    page.foot = vec![
        cx.button(PanelButton::Copy),
        cx.button_as(PanelButton::Pin, &pin),
        cx.button(PanelButton::Delete).tone(Tone::Danger),
        insert.tone(Tone::Primary),
    ];
    page
}

/// A dictated text: its line names when, the language by its code and the model; readers also hear its tick and pin.
fn history_row(t: &I18n, r: &HistoryRow) -> HistoryView {
    let text = clean(r.text.expose());
    let lang = t.name_of(LANG_NAME, &clean(&r.lang));
    let args = [
        (WHEN, t.clip_when(&r.when)),
        (LANG, lang),
        (MODEL, clean(r.model.expose())),
    ];
    let meta = t.voice_with(VOICE_META, &args);
    let ticked = r.tick.map(|n| t.counted(VOICE_TICKED, usize::from(n)));
    let pinned = r.pinned.then(|| t.text(VOICE_PINNED));
    let parts: Vec<String> = [Some(text.clone()), Some(meta.clone()), ticked, pinned]
        .into_iter()
        .flatten()
        .collect();
    HistoryView {
        id: r.id,
        text,
        rtl: r.rtl,
        arabic: r.arabic,
        meta,
        pinned: r.pinned,
        tick: r.tick,
        name: parts.join(&t.text(VOICE_JOIN)),
    }
}
