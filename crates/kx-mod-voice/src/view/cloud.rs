//! The Cloud page and a service's card (ADR-0047): the `cloud` switch, then the services offered, Groq first, each with
//! its mark, free allowance, warning and sealed key (edge 24), dimmed while cloud is off (edge 21). A card shows its facts with
//! the day read, old past `facts_days` (edge 15), and asks again for newer terms (edge 16).

use kx_module_api::ports::{
    CardPart, CloudButton, CloudSheet, Pages, ServiceMark, ServiceRow, VoiceSheet, VoiceTab,
};

use super::{Facts, short};
use crate::panel::{Key, Page, Panel};
use crate::services::Service;

impl Panel {
    /// The Cloud page's shown services; while cloud is off they show dimmed, untickable, each sealed key still removable.
    pub(super) fn cloud_sheet(&mut self, facts: &Facts<'_>) -> (VoiceSheet, Pages) {
        let on = facts.cloud.is_some_and(|c| c.on(facts.settings));
        let ticks = self.ticks.clone();
        let rows = facts.cloud.map(|c| c.listed()).unwrap_or_default();
        let list = marked(facts, rows).into_iter().map(|(row, mark)| {
            let id = row.id.clone();
            let key = if on {
                Key::Service(id)
            } else {
                Key::Dimmed(id)
            };
            let shown = ServiceRow {
                name: row.shown(),
                mark,
                untested: row.untested,
                free: row.name.is_some().then(|| row.facts.free.clone()),
                warn: row.facts.warn.as_ref().map(|f| f.id.clone()),
                remove: removable(facts, row),
                tick: ticks.contains(&key),
            };
            (key, shown)
        });
        let (rows, pages) = self.paged(list.collect());
        let acts = self.cloud_acts(facts);
        (VoiceSheet::Cloud(CloudSheet { on, rows, acts }), pages)
    }

    /// The shown service's card on today's date; it asks while the service is off or its newer terms wait.
    pub(super) fn card_sheet(&mut self, facts: &Facts<'_>) -> (VoiceSheet, Pages) {
        let (Some(cloud), Some(row)) = (facts.cloud, self.shown_row(facts)) else {
            return self.cloud_sheet(facts);
        };
        let mut card = cloud.card(facts.settings, row, facts.now.date);
        let (part, pages) = self.parted(&CardPart::ALL);
        card.part = part.unwrap_or(card.part);
        if self.asking(facts) {
            card.on = None;
        }
        (VoiceSheet::Card(Box::new(card)), pages)
    }

    /// Part `at` of `parts`, one to a page of the panel, kept within them, as a card or a key page shows no rows.
    pub(super) fn parted<T: Copy>(&mut self, parts: &[T]) -> (Option<T>, Pages) {
        self.shown.clear();
        let count = parts.len().max(1);
        self.at = self.at.min(count - 1);
        let pages = Pages {
            at: short(self.at),
            count: short(count),
        };
        (parts.get(self.at).copied(), pages)
    }

    /// The cloud buttons the shown page offers now: the ticked service's on the Cloud page, the link on a card, and
    /// the key page's.
    pub(crate) fn cloud_acts(&self, facts: &Facts<'_>) -> Vec<CloudButton> {
        match self.page {
            Some(Page::Tab(VoiceTab::Cloud)) => self.ticked_acts(facts),
            Some(Page::Card) => {
                let site = self.shown_row(facts).and_then(|r| r.facts.source.as_ref());
                site.map(|_| CloudButton::Link).into_iter().collect()
            }
            Some(Page::Lines) => self.lines_acts(facts),
            _ => Vec::new(),
        }
    }

    /// The Cloud page's footer for the ticked service: Facts always, Key and model and Switch off once on, Switch on
    /// while off or while newer terms wait.
    fn ticked_acts(&self, facts: &Facts<'_>) -> Vec<CloudButton> {
        let Some(Key::Service(id)) = self.ticked() else {
            return Vec::new();
        };
        let Some((row, mark)) = Self::services(facts).into_iter().find(|(r, _)| &r.id == id) else {
            return Vec::new();
        };
        let asks = facts.cloud.is_some_and(|c| c.asks(facts.settings, row));
        let on = mark != ServiceMark::Off;
        let mut acts = vec![CloudButton::Facts];
        if on {
            acts.extend([CloudButton::Lines, CloudButton::Off]);
        }
        acts.extend((!on || asks).then_some(CloudButton::On));
        acts
    }

    /// The services the Cloud page offers, in order, each with its mark.
    pub(crate) fn services<'a>(facts: &Facts<'a>) -> Vec<(&'a Service, ServiceMark)> {
        let rows = facts.cloud.map(|c| c.offered(facts.settings));
        marked(facts, rows.unwrap_or_default())
    }

    /// The row of the service the card or key page shows, while cloud offers it.
    pub(crate) fn shown_row<'a>(&self, facts: &Facts<'a>) -> Option<&'a Service> {
        let id = self.cloud.shown.as_deref()?;
        let cloud = facts.cloud?;
        cloud
            .offered(facts.settings)
            .into_iter()
            .find(|r| r.id == id)
    }
}

/// Each of `rows` with its mark, Off for every row while cloud is off.
fn marked<'a>(facts: &Facts<'a>, rows: Vec<&'a Service>) -> Vec<(&'a Service, ServiceMark)> {
    let Some(cloud) = facts.cloud else {
        return Vec::new();
    };
    let s = facts.settings;
    let mark = |row: &'a Service| (row, cloud.mark(s, row, facts.keyed.contains(&row.id)));
    rows.into_iter().map(mark).collect()
}

/// True when `row` has its own Remove key: a key is sealed while its key page is out of reach, as cloud services or the
/// service are off (owner, 2026-11-20).
pub(crate) fn removable(facts: &Facts<'_>, row: &Service) -> bool {
    let keyed = facts.keyed.contains(&row.id);
    let mark = facts.cloud.map(|c| c.mark(facts.settings, row, keyed));
    keyed && mark == Some(ServiceMark::Off)
}
