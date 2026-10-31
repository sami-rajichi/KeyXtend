//! The page the panel draws (spec §7 "Panel"): the shown page's rows with their tick numbers, times, pins and expiry
//! chips, then the search, pages, preview, question, storage line and note. It holds no words, as the ui words it;
//! an empty title asks the ui for the kind's name.

use kx_module_api::ports::{
    ClipKind, ClipPage, ClipPart, ClipRow, ClipView, Date, LocalTime, Pages, PanelNote, Pixels,
    Preview, Room, When,
};
use kx_module_api::{Redacted, is_arabic, is_rtl};

use crate::entry::{Entry, Stamp, text_of};
use crate::keep::{Limits, leaves};
use crate::order::History;
use crate::panel::{Panel, unpinned};

/// What the view shows beyond the panel and the history; the module gathers it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    /// The wall time now, for the rows' days and the expiry chips.
    pub now: LocalTime,
    /// What the history takes and its drive has free, once read.
    pub room: Option<Room>,
    /// The note to show, if any.
    pub note: Option<PanelNote>,
}

/// The view of `panel`'s page over `history`, with `facts` and `limits`; `thumb` gives a row's saved thumbnail and is
/// asked only for rows that have one. It counts a new round, so a click names the page it saw.
pub fn build(
    panel: &mut Panel,
    history: &History,
    facts: &Facts,
    limits: &Limits,
    thumb: impl Fn(&Entry) -> Option<Pixels>,
) -> ClipView {
    panel.sync(history);
    let list = panel.list(history);
    let per = panel.rows();
    let hi = panel.highlight(&list);
    let at = hi.unwrap_or_default() / per;
    let shown: Vec<&Entry> = list.iter().skip(at * per).take(per).copied().collect();
    let ticks = panel.ticks();
    let rows = shown.iter().map(|e| row(e, ticks, facts, limits, &thumb));
    let rows: Vec<ClipRow> = rows.collect();
    let round = panel.shown(shown.iter().map(|e| e.id).collect());
    ClipView::new(ClipPage {
        round,
        rows,
        highlight: hi.map(|h| u8::try_from(h - at * per).unwrap_or(u8::MAX)),
        filter: panel.filter(),
        search: panel.query().clone(),
        pages: Pages {
            at: short(at),
            count: short(list.len().div_ceil(per).max(1)),
        },
        ticked: short(panel.ticks().len()),
        unpin: panel.unpins(history),
        preview: panel.preview().map(|(_, p)| p.clone()),
        confirm: panel.asked().map(|ids| short(ids.len())),
        unpinned: short(unpinned(history).len()),
        room: facts.room.clone(),
        cap: limits.max_bytes,
        note: facts.note,
        kept_days: limits.keep_days,
        empty: history.is_empty(),
    })
}

/// The preview of `entry`, whose content is `parts`: its text cut to `chars` characters with the whole count, its
/// files' paths, or its image, which the module draws later.
#[must_use]
pub fn preview(entry: &Entry, parts: &[ClipPart], chars: usize) -> Preview {
    match entry.kind {
        ClipKind::Image => Preview::Image(None),
        ClipKind::File | ClipKind::Media => Preview::Files(entry.files.clone()),
        ClipKind::Text | ClipKind::Link => {
            let text = text_of(parts).unwrap_or_default();
            let count = text.chars().count();
            Preview::Text {
                text: Redacted::new(text.chars().take(chars).collect()),
                chars: u32::try_from(count).unwrap_or(u32::MAX),
            }
        }
    }
}

/// The row of `entry`, with its place in `ticks`, as seen at `facts.now`.
fn row(
    entry: &Entry,
    ticks: &[u64],
    facts: &Facts,
    limits: &Limits,
    thumb: &impl Fn(&Entry) -> Option<Pixels>,
) -> ClipRow {
    let title = entry.title.expose();
    let app = entry.app.as_ref().filter(|a| !a.name.is_empty());
    let tick = ticks.iter().position(|&t| t == entry.id);
    ClipRow {
        id: entry.id,
        kind: entry.kind,
        title: entry.title.clone(),
        rtl: is_rtl(title),
        arabic: is_arabic(title),
        app: app.map(|a| Redacted::new(a.name.clone())),
        when: when(entry.at, &facts.now),
        bytes: entry.shown,
        tick: tick.map(|n| short(n + 1)),
        pinned: entry.pinned(),
        expires: expires(entry, facts.now.date, limits),
        thumb: if entry.thumb { thumb(entry) } else { None },
    }
}

/// When `at` was, in its own local time, in calendar days before `now`; a time after now, from a clock set back,
/// reads today.
fn when(at: Stamp, now: &LocalTime) -> When {
    let local = at.local().unwrap_or(*now);
    let days = now.date.days_since(local.date).max(0);
    When {
        days_ago: u32::try_from(days).unwrap_or(u32::MAX),
        date: local.date,
        hour: local.hour,
        minute: local.minute,
    }
}

/// True when `entry` is media the sweep after `today` takes out, so it leaves at the end of today.
fn expires(entry: &Entry, today: Date, limits: &Limits) -> bool {
    let tomorrow = Date::from_day_number(today.day_number().saturating_add(1));
    entry.kind == ClipKind::Media && tomorrow.is_some_and(|t| leaves(entry, t, limits))
}

/// `n` as a count the page holds, or the largest one.
fn short(n: usize) -> u16 {
    u16::try_from(n).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests;
