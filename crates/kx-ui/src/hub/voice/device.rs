//! The Device page as QML reads it: Automatic, the processor with its cores and memory, each card or built-in graphics
//! with its memory and check, and the latest failed check's band with its reason.

use super::panel::{Cx, Page, Sheet};
use super::parts::{Hint, TOP_FAILED, Tone, Top};
use super::sheets::{mark, mark_key, tab_id};
use crate::I18n;
use crate::i18n::clean;
use crate::i18n::keys::vars::{ALLOWED, CORES, FASTER, FREE, NAME, NEED, TOTAL, WHY, WORDS};
use crate::i18n::keys::{
    VOICE_AUTO, VOICE_AUTO_LINE, VOICE_BUILT_IN_LINE, VOICE_CARD_LINE, VOICE_CHECKING, VOICE_CORES,
    VOICE_CPU, VOICE_CPU_LINE, VOICE_DEVICE_HINT, VOICE_FAILED, VOICE_FASTER, VOICE_TIMES,
    VOICE_USE_DEVICE, VOICE_WHY_DIFFERS, VOICE_WHY_ERROR, VOICE_WHY_ROOM, VOICE_WHY_SLOW,
    VOICE_WORDS, VoiceMark, mark_id,
};
use kx_module_api::ports::{
    CardCheck, DeviceItem, DeviceRow, DeviceSheet, MAX_PERCENT, Memory, PanelButton, Verdict,
    VoiceTab,
};
use serde::Serialize;

/// A Device row's kind, which picks its icon: Automatic, the processor, a card, built-in graphics.
const AUTO_KIND: &str = "auto";
const CPU_KIND: &str = "cpu";
const CARD_KIND: &str = "card";
const BUILT_IN_KIND: &str = "built-in";

/// A Device row.
#[derive(Serialize)]
pub(super) struct DeviceView {
    kind: &'static str,
    name: String,
    line: String,
    mark: Option<Hint>,
    check: Option<Check>,
    tick: bool,
    used: bool,
}

/// A card's check while it runs, its share from 0 to 1.
#[derive(Serialize)]
pub(super) struct Check {
    line: String,
    share: f64,
}

/// The Device page: the latest failed check's band, the rows, then Check and Use.
pub(super) fn page(cx: &Cx<'_>, d: &DeviceSheet) -> Page {
    let t = cx.t;
    let sheet = Sheet::Device {
        rows: d.rows.iter().map(|r| row(t, r)).collect(),
    };
    let tab = VoiceTab::Device;
    let mut page = Page::new(tab_id(tab), cx.tab(tab), t.text(VOICE_DEVICE_HINT), sheet);
    page.top = d.failed.as_ref().map(|f| {
        let args = [(NAME, clean(&f.name)), (WHY, why(t, f.verdict))];
        Top::new(TOP_FAILED, t.voice_with(VOICE_FAILED, &args), None)
    });
    let used = cx.button_as(PanelButton::Use, VOICE_USE_DEVICE);
    page.foot = vec![cx.button(PanelButton::Check), used.tone(Tone::Primary)];
    page
}

/// What a Device row shows: its kind, name and line, and a card's check.
type Item = (&'static str, String, String, Option<CardCheck>);

/// Device row `r`: a running check shows over any mark.
fn row(t: &I18n, r: &DeviceRow) -> DeviceView {
    let (kind, name, line, check) = item(t, &r.item);
    let running = match check {
        Some(CardCheck::Running { percent }) => Some(Check {
            line: t.text(VOICE_CHECKING),
            share: f64::from(percent.min(MAX_PERCENT)) / f64::from(MAX_PERCENT),
        }),
        _ => None,
    };
    let mark = running.is_none().then(|| row_mark(t, r.used, check));
    DeviceView {
        kind,
        name,
        line,
        mark: mark.flatten(),
        check: running,
        tick: r.tick,
        used: r.used,
    }
}

/// Device `item`'s kind, name and line, with its memory, and a card's check.
fn item(t: &I18n, item: &DeviceItem) -> Item {
    let memory = |m: &Memory| {
        [
            (TOTAL, t.voice_memory(m.total_mb)),
            (FREE, t.voice_memory(m.free_mb)),
        ]
    };
    match item {
        DeviceItem::Auto => (AUTO_KIND, t.text(VOICE_AUTO), t.text(VOICE_AUTO_LINE), None),
        DeviceItem::Cpu { cores, memory: m } => {
            let [total, free] = memory(m);
            let cores = (CORES, t.counted(VOICE_CORES, usize::from(*cores)));
            let line = t.voice_with(VOICE_CPU_LINE, &[cores, total, free]);
            (CPU_KIND, t.text(VOICE_CPU), line, None)
        }
        DeviceItem::Card {
            name,
            built_in,
            memory: m,
            check,
        } => {
            let (kind, key) = if *built_in {
                (BUILT_IN_KIND, VOICE_BUILT_IN_LINE)
            } else {
                (CARD_KIND, VOICE_CARD_LINE)
            };
            (
                kind,
                clean(name),
                t.voice_with(key, &memory(m)),
                Some(*check),
            )
        }
    }
}

/// A Device row's mark: In use, else its check's verdict, if any.
fn row_mark(t: &I18n, used: bool, check: Option<CardCheck>) -> Option<Hint> {
    match check {
        _ if used => Some(mark(t, VoiceMark::Used)),
        Some(CardCheck::Done(Verdict::Passed { gain })) => {
            let faster = (FASTER, t.voice_times(VOICE_FASTER, gain));
            let text = t.voice_with(&mark_key(VoiceMark::Passed), &[faster]);
            Some(Hint::new(mark_id(VoiceMark::Passed), text))
        }
        Some(CardCheck::Done(_)) => Some(mark(t, VoiceMark::Failed)),
        _ => None,
    }
}

/// Why a check failed, with its numbers.
fn why(t: &I18n, verdict: Verdict) -> String {
    let words = |n: u16| t.counted(VOICE_WORDS, usize::from(n));
    match verdict {
        Verdict::Slow { gain, need } => {
            let faster = (FASTER, t.voice_times(VOICE_FASTER, gain));
            let need = (NEED, t.voice_times(VOICE_TIMES, need));
            t.voice_with(VOICE_WHY_SLOW, &[faster, need])
        }
        Verdict::Differs { words: n, allowed } => {
            let args = [(WORDS, words(n)), (ALLOWED, words(u16::from(allowed)))];
            t.voice_with(VOICE_WHY_DIFFERS, &args)
        }
        Verdict::NoRoom => t.text(VOICE_WHY_ROOM),
        Verdict::Failed => t.text(VOICE_WHY_ERROR),
        Verdict::Passed { .. } => String::new(),
    }
}
