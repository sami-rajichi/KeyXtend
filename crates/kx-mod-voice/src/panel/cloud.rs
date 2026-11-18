//! The cloud pages' state and clicks (ADR-0047): the Cloud page's switch and its ticked service's buttons, a card's
//! confirm, and the writes each act leaves in `[voice]`. The key page's acts are in `lines`, the Models page's sides in
//! `sides`; `services` is written only through `VoiceSettings::services_value`.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    AddressFault, CloudButton, KeyCheck, Link, ModelSide, ServiceMark, VoiceClick, VoiceTab,
};

use super::typing::Typed;
use super::{Do, Key, Page, Panel};
use crate::cloud::Cloud;
use crate::settings::{CLOUD, PICKS, SERVICES, VoiceSettings};
use crate::view::Facts;

/// The cloud pages' state.
#[derive(Clone, Debug)]
pub(crate) struct Clouds {
    /// The service the card or the key page shows.
    pub(crate) shown: Option<String>,
    /// The card was opened from the key page, so Back goes there.
    pub(crate) from_lines: bool,
    /// The shown key's characters, read as its page opens.
    pub(crate) dots: u32,
    /// How the shown key's test stands.
    pub(crate) check: KeyCheck,
    /// Counts Pastes, tests and key changes, so an answer to an older one is dropped.
    pub(crate) round: u32,
    /// Why a Paste or a typed key kept nothing, Busy or Unfit, until the next round; the test's answer stays beneath.
    pub(crate) unkept: Option<KeyCheck>,
    /// The Paste waited for.
    pub(crate) take: Option<u32>,
    /// The test waited for.
    pub(crate) test: Option<u32>,
    /// The address just refused, with why, until the page changes.
    pub(crate) refused: Option<(Redacted<String>, AddressFault)>,
    /// The Other model id just refused, until the page changes.
    pub(crate) refused_model: Option<Redacted<String>>,
    /// The Models page's side, and the page the other side was left on.
    pub(crate) side: (ModelSide, usize),
    /// The key page's lines as typed (ADR-0049).
    pub(crate) typed: Typed,
}

impl Default for Clouds {
    fn default() -> Self {
        Self {
            shown: None,
            from_lines: false,
            dots: 0,
            check: KeyCheck::Untested,
            round: 0,
            unkept: None,
            take: None,
            test: None,
            refused: None,
            refused_model: None,
            side: (ModelSide::Local, 0),
            typed: Typed::default(),
        }
    }
}

impl Clouds {
    /// Starts a new round, so the Paste or test waited for is dropped, a dropped test leaving the key untested; gives
    /// its number.
    pub(crate) fn next(&mut self) -> u32 {
        self.round = self.round.wrapping_add(1);
        if self.test.take().is_some() {
            self.check = KeyCheck::Untested;
        }
        (self.take, self.unkept) = (None, None);
        self.round
    }
}

impl Panel {
    /// A click of the cloud pages, the Models page's sides, or the Cloud page opened from Home or Models.
    pub(super) fn cloud_click(&mut self, click: &VoiceClick, facts: &Facts<'_>) -> Vec<Do> {
        let lines = self.page == Some(Page::Lines);
        match click {
            VoiceClick::Open(VoiceTab::Cloud) => self.open_cloud(),
            VoiceClick::Cloud(on) if self.page == Some(Page::Tab(VoiceTab::Cloud)) => {
                self.ticks.clear();
                edit(facts, |c, s| {
                    c.switch(s, *on);
                    true
                })
            }
            VoiceClick::Service(button) if self.cloud_acts(facts).contains(button) => {
                self.cloud_button(*button, facts)
            }
            VoiceClick::SetKey(key) if lines => self.set_key(key),
            VoiceClick::SetAddress(text) => self.type_address(text, facts),
            VoiceClick::SetModel(text) => self.type_model(text, facts),
            VoiceClick::Save(line) => self.save(*line, facts),
            VoiceClick::Caret { line, at } => {
                self.place_caret(*line, *at.expose(), facts);
                Vec::new()
            }
            VoiceClick::KeepNothing(on) => self.keep_nothing(*on, facts),
            VoiceClick::Side(side) => self.side(*side),
            _ => Vec::new(),
        }
    }

    /// Opens the Cloud page from Home, or from the Models page's band.
    fn open_cloud(&mut self) -> Vec<Do> {
        let from = matches!(self.page, Some(Page::Home | Page::Tab(VoiceTab::Models)));
        if from {
            return self.go(Page::Tab(VoiceTab::Cloud));
        }
        Vec::new()
    }

    /// A cloud button the shown page offers now.
    fn cloud_button(&mut self, button: CloudButton, facts: &Facts<'_>) -> Vec<Do> {
        let ticked = match self.ticked() {
            Some(Key::Service(id)) if self.page == Some(Page::Tab(VoiceTab::Cloud)) => Some(id),
            _ => self.cloud.shown.as_ref(),
        };
        let Some(id) = ticked.cloned() else {
            return Vec::new();
        };
        match button {
            CloudButton::On | CloudButton::Facts => {
                self.cloud.from_lines = self.page == Some(Page::Lines);
                self.show(id, Page::Card)
            }
            CloudButton::Lines => self.show(id, Page::Lines),
            CloudButton::Off => self.turn_off(&id, facts),
            CloudButton::Link => self.link(&id, facts),
            CloudButton::Paste | CloudButton::Test | CloudButton::Remove => {
                self.key_button(&id, button, facts)
            }
        }
    }

    /// Shows service `id`'s card or key page; another service's key starts untested.
    fn show(&mut self, id: String, page: Page) -> Vec<Do> {
        if self.cloud.shown.as_ref() != Some(&id) {
            self.cloud.check = KeyCheck::Untested;
        }
        self.cloud.shown = Some(id);
        self.go(page)
    }

    /// Switches service `id` off: its picks go to the local default (edge 16); its sealed key stays, as only Remove key
    /// deletes it.
    fn turn_off(&mut self, id: &str, facts: &Facts<'_>) -> Vec<Do> {
        let mut out = edit(facts, |c, s| {
            c.turn_off(s, id);
            true
        });
        if self.page != Some(Page::Tab(VoiceTab::Cloud)) {
            out.extend(self.go(Page::Tab(VoiceTab::Cloud)));
        }
        out
    }

    /// The shown page's link: the card's facts page, or the key page's where keys are made; only the row's own HTTPS
    /// address from the table, never a typed one (edge 23).
    fn link(&self, id: &str, facts: &Facts<'_>) -> Vec<Do> {
        let row = facts.cloud.and_then(|c| c.row(id));
        let url = match self.page {
            Some(Page::Card) => row.and_then(|r| r.facts.source.as_ref()),
            _ => row.and_then(|r| r.keys.as_ref()),
        };
        let link = url.and_then(|u| Link::new(u.expose().to_owned()).ok());
        link.map(Do::Link).into_iter().collect()
    }

    /// An answer to the card's question: yes switches the service on at today's terms and opens its key page; no to
    /// newer terms switches it off (edge 16), and no to a service still off goes back.
    pub(super) fn card_confirm(&mut self, yes: bool, facts: &Facts<'_>) -> Vec<Do> {
        let (Some(id), Some(cloud)) = (self.cloud.shown.clone(), facts.cloud) else {
            return Vec::new();
        };
        let Some(row) = cloud.row(&id).filter(|_| self.asking(facts)) else {
            return Vec::new();
        };
        if yes {
            let today = facts.now.date;
            let mut out = edit(facts, |c, s| c.confirm(s, &id, today));
            if !out.is_empty() {
                out.extend(self.go(Page::Lines));
            }
            return out;
        }
        if cloud.asks(facts.settings, row) {
            return self.turn_off(&id, facts);
        }
        let back = if self.cloud.from_lines {
            Page::Lines
        } else {
            Page::Tab(VoiceTab::Cloud)
        };
        self.go(back)
    }

    /// True while the shown card asks: its service is off, or its newer terms wait to be confirmed.
    pub(crate) fn asking(&self, facts: &Facts<'_>) -> bool {
        let id = self.cloud.shown.as_deref();
        let (Some(cloud), Some(id)) = (facts.cloud, id) else {
            return false;
        };
        let s = facts.settings;
        cloud
            .row(id)
            .is_some_and(|row| cloud.mark(s, row, false) == ServiceMark::Off || cloud.asks(s, row))
    }
}

/// The writes `act` leaves, when it changed `[voice]` and the cloud rules load; none for a refused act.
pub(super) fn edit(
    facts: &Facts<'_>,
    act: impl FnOnce(&Cloud<'_>, &mut VoiceSettings) -> bool,
) -> Vec<Do> {
    let Some(cloud) = facts.cloud else {
        return Vec::new();
    };
    let mut next = facts.settings.clone();
    if !act(&cloud, &mut next) {
        return Vec::new();
    }
    writes(facts.settings, &next)
}

/// The writes that bring `[voice]` from `before` to `after`: the picks, the services and the switch that changed.
fn writes(before: &VoiceSettings, after: &VoiceSettings) -> Vec<Do> {
    let mut out = Vec::new();
    if before.picks != after.picks {
        let picks = toml::Value::try_from(&after.picks).ok();
        out.extend(picks.map(|v| Do::Set(PICKS, v)));
    }
    if before.services != after.services {
        out.extend(after.services_value().map(|v| Do::Set(SERVICES, v)));
    }
    if before.cloud != after.cloud {
        out.push(Do::Set(CLOUD, toml::Value::Boolean(after.cloud)));
    }
    out
}
