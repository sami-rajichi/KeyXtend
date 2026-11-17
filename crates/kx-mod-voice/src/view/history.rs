//! The History page: the texts, pinned first then newest first, each with its time, language, model and service,
//! ticked in order; the counts its header, Pin and Delete all need. It lists what is stored even with `history` off.

use kx_module_api::ports::{
    HistoryRow, HistorySheet, LocalTime, Pages, PanelButton, VoiceSheet, When,
};
use kx_module_api::{is_arabic, is_rtl};

use super::{Facts, service_of, shown_name};
use crate::history::{Asked, Entry, Stamp};
use crate::panel::{Key, Panel};

impl Panel {
    /// The History page's shown rows.
    pub(super) fn history_sheet(&mut self, facts: &Facts<'_>) -> (VoiceSheet, Pages) {
        let history = facts.history;
        let ticks = self.ticks.clone();
        let list = history.entries().iter().map(|e| {
            let key = Key::Text(e.id);
            let tick = ticks.iter().position(|k| *k == key);
            (key, row(e, tick, &facts.now))
        });
        let (rows, pages) = self.paged(list.collect());
        let unpinned = self.asked.as_ref().map_or(history.unpinned(), Asked::count);
        let sheet = HistorySheet {
            rows,
            total: count(history.len()),
            unpinned: count(unpinned),
            unpin: self.unpins(facts),
            confirm: self.asked.is_some(),
        };
        (VoiceSheet::History(sheet), pages)
    }

    /// History's buttons that act now: the footer's with a tick, Delete all with an unpinned text; none while asking.
    pub(crate) fn history_acts(&self, facts: &Facts<'_>) -> Vec<PanelButton> {
        if self.asked.is_some() {
            return Vec::new();
        }
        let mut acts = Vec::new();
        if !self.ticks.is_empty() {
            acts.extend([
                PanelButton::Insert,
                PanelButton::Copy,
                PanelButton::Pin,
                PanelButton::Delete,
            ]);
        }
        acts.extend((facts.history.unpinned() > 0).then_some(PanelButton::DeleteAll));
        acts
    }

    /// The ticked texts, in tick order.
    pub(crate) fn ticked_texts(&self) -> Vec<u64> {
        let id = |k: &Key| match k {
            Key::Text(id) => Some(*id),
            _ => None,
        };
        self.ticks.iter().filter_map(id).collect()
    }

    /// True when Pin reads Unpin: every ticked text is pinned, and one is ticked.
    pub(crate) fn unpins(&self, facts: &Facts<'_>) -> bool {
        let ids = self.ticked_texts();
        let pinned = |id: &u64| facts.history.get(*id).is_some_and(|e| e.pinned);
        !ids.is_empty() && ids.iter().all(pinned)
    }
}

/// The row of `entry`, at place `tick` of the tick order, as seen `now`.
fn row(entry: &Entry, tick: Option<usize>, now: &LocalTime) -> HistoryRow {
    let text = entry.text.expose();
    HistoryRow {
        id: entry.id,
        text: entry.text.clone(),
        rtl: is_rtl(text),
        arabic: is_arabic(text),
        when: when(entry.at, now),
        lang: entry.lang.clone(),
        model: shown_name(&entry.model),
        cloud: service_of(&entry.model),
        pinned: entry.pinned,
        tick: tick.map(|n| u16::try_from(n + 1).unwrap_or(u16::MAX)),
    }
}

/// When `at` was, in its own local time, in calendar days before `now`; a time after now reads today.
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

/// `n` as a count the page holds.
fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}
