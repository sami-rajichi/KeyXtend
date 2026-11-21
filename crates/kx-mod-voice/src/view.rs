//! The Voice panel's view (design "The gear and the Voice panel"): each page's rows, `page_rows` to a page with the
//! ticks shown, the buttons that act now and the note band. It holds no words; the ui words each value.

mod cloud;
mod compare;
mod device;
mod history;
mod home;
mod lines;
mod models;
mod remote;

use std::collections::BTreeSet;
use std::mem;

use kx_module_api::ports::{
    BarHint, Device, DeviceKind, Drive, FolderList, LayoutChanged, LocalTime, OptionRow,
    OptionValue, Pages, PanelButton, ServiceName, VoiceLang, VoicePanel, VoiceSheet, VoiceTab,
};
use kx_module_api::{Redacted, is_arabic};

use crate::VoiceSettings;
use crate::cloud::{Choice, Cloud, cloud_pick};
use crate::counts::{Checks, Scores};
use crate::history::History;
use crate::panel::{Key, Page, Panel, options};
use crate::services::{self, Service};

pub(crate) use cloud::removable;

/// What the view and the clicks need beyond the panel; the actor holds it.
#[derive(Copy, Clone, Debug)]
pub struct Facts<'a> {
    /// `[voice]`.
    pub settings: &'a VoiceSettings,
    /// The layout the keyboard shows, whose language the pages are for.
    pub front: &'a LayoutChanged,
    /// The fixed drives and their room, read just before.
    pub drives: &'a [Drive],
    /// The processor and the cards, as last read.
    pub devices: &'a [Device],
    /// The check clips' languages the speech port ships, in the order a model takes them.
    pub clips: &'a [String],
    /// Each card's check.
    pub checks: &'a Checks,
    /// Compare's last numbers per language.
    pub scores: &'a Scores,
    /// History.
    pub history: &'a History,
    /// The wall time now.
    pub now: LocalTime,
    /// Try again can act (`Flow::can_again`).
    pub again: bool,
    /// A dictation listens or writes, so Compare waits.
    pub dictating: bool,
    /// The cloud rules, in a run that is hidden or not; `None` when the services table does not load, so cloud is off.
    pub cloud: Option<Cloud<'a>>,
    /// The services with a key, from the blobs' names alone.
    pub keyed: &'a BTreeSet<String>,
}

impl Facts<'_> {
    /// The language in front as the panel names it.
    #[must_use]
    pub fn lang(&self) -> VoiceLang {
        VoiceLang {
            name: self.front.name.clone(),
            iso: self.front.iso.clone(),
            rtl: self.front.rtl,
            arabic: is_arabic(&self.front.name),
        }
    }

    /// The language in front, an ISO 639 code; empty when unknown.
    #[must_use]
    pub fn iso(&self) -> &str {
        &self.front.iso
    }

    /// The processor as last read; `None` before the first read.
    #[must_use]
    pub fn cpu(&self) -> Option<&Device> {
        self.devices.iter().find(|d| d.kind == DeviceKind::Cpu)
    }
}

impl<'a> Facts<'a> {
    /// The language in front's cloud pick while the rules take it; it then dictates with that, not a local model.
    #[must_use]
    pub fn cloud_choice(&self) -> Option<Choice<'a>> {
        self.cloud?.choice(self.settings, &self.front.iso)
    }
}

impl Panel {
    /// The panel to draw while it is open; it notes each row's meaning, so a click names what it saw.
    pub fn view(&mut self, facts: &Facts<'_>) -> Option<VoicePanel> {
        let page = self.page()?;
        self.sync(facts);
        let (sheet, pages) = match page {
            Page::Home => {
                let list = VoiceTab::ALL.into_iter().map(|t| (Key::Plain, t)).collect();
                let (tabs, pages) = self.paged(list);
                (VoiceSheet::Home(home::home(self, facts, tabs)), pages)
            }
            Page::Tab(VoiceTab::Cloud) => self.cloud_sheet(facts),
            Page::Card => self.card_sheet(facts),
            Page::Lines => self.lines_sheet(facts),
            Page::Tab(VoiceTab::Models) => self.models_sheet(facts),
            Page::Tab(VoiceTab::Device) => self.device_sheet(facts),
            Page::Tab(VoiceTab::Compare) => self.compare_sheet(facts),
            Page::Tab(VoiceTab::History) => self.history_sheet(facts),
            Page::Tab(VoiceTab::Options) => self.options_sheet(facts),
            Page::Folders => self.folders_sheet(facts),
        };
        Some(VoicePanel {
            sheet,
            lang: facts.lang(),
            pages,
            ticked: short(self.ticks.len()),
            acts: self.acts(facts),
            note: self.band(page),
        })
    }

    /// The page's buttons that act now.
    pub(crate) fn acts(&self, facts: &Facts<'_>) -> Vec<PanelButton> {
        let mut acts = vec![PanelButton::Close];
        match self.page() {
            Some(Page::Home) => acts.extend(facts.again.then_some(PanelButton::Again)),
            Some(Page::Tab(VoiceTab::Models)) => acts.extend(self.models_acts(facts)),
            Some(Page::Tab(VoiceTab::Device)) => acts.extend(self.device_acts(facts)),
            Some(Page::Tab(VoiceTab::Compare)) => acts.extend(self.compare_acts()),
            Some(Page::Tab(VoiceTab::History)) => acts.extend(self.history_acts(facts)),
            Some(Page::Folders) => acts.extend(self.browse.as_ref().map(|_| PanelButton::Here)),
            // The cloud pages' buttons are their sheets' own.
            Some(Page::Tab(VoiceTab::Options | VoiceTab::Cloud) | Page::Card | Page::Lines)
            | None => {}
        }
        if self.page().is_some_and(|p| p != Page::Home) {
            acts.insert(0, PanelButton::Back);
        }
        acts
    }

    /// The shown page of `list`, `page_rows` to a page, kept within it; notes what each shown row stands for.
    pub(crate) fn paged<T>(&mut self, list: Vec<(Key, T)>) -> (Vec<T>, Pages) {
        self.grouped(list.into_iter().map(|row| vec![row]).collect())
    }

    /// As `paged`, but a group that fits a page never splits: it starts the next page instead. The page holding the
    /// held row shows, so a click that moves its row keeps it on screen.
    pub(crate) fn grouped<T>(&mut self, groups: Vec<Vec<(Key, T)>>) -> (Vec<T>, Pages) {
        let mut pages = lay_out(groups, self.rows.max(1));
        let holds = |p: &Vec<(Key, T)>| p.iter().any(|(k, _)| self.hold.as_ref() == Some(k));
        let held = pages.iter().position(holds);
        let count = pages.len().max(1);
        let at = held.unwrap_or(self.at).min(count - 1);
        self.at = at;
        let page = pages.get_mut(at).map(mem::take).unwrap_or_default();
        let (keys, rows): (Vec<Key>, Vec<T>) = page.into_iter().unzip();
        self.shown = keys;
        let pages = Pages {
            at: short(at),
            count: short(count),
        };
        (rows, pages)
    }

    /// One page and no rows, as the set text, a card and a key page.
    pub(crate) fn single(&mut self) -> Pages {
        self.shown.clear();
        Pages { at: 0, count: 1 }
    }

    /// The note band: the panel's own, else Compare's on its page.
    fn band(&self, page: Page) -> Option<BarHint> {
        let compare = (page == Page::Tab(VoiceTab::Compare)).then(|| self.compare.note_shown());
        self.note.clone().or(compare.flatten())
    }

    /// Drops the ticks of texts History no longer holds, and the question once nothing is left to delete.
    fn sync(&mut self, facts: &Facts<'_>) {
        let held = |k: &Key| !matches!(k, Key::Text(id) if facts.history.get(*id).is_none());
        self.ticks.retain(held);
        if facts.history.ask_all().is_none() {
            self.asked = None;
        }
    }

    /// The Options page's shown rows; a pair whose second name is chosen has its hint on the row below, on its page.
    fn options_sheet(&mut self, facts: &Facts<'_>) -> (VoiceSheet, Pages) {
        let table = options::values(facts.settings);
        let cores = facts.cpu().map(|d| d.cores);
        let rows = |(i, o): (usize, &'static options::Opt)| {
            let row = o.row(&table, cores)?;
            let second = matches!(row.value, OptionValue::Pair { second: true, .. });
            let hint = OptionRow {
                value: OptionValue::Hint,
                ..row
            };
            let group = [(Key::Opt(i), row)]
                .into_iter()
                .chain(second.then_some((Key::Plain, hint)));
            Some(group.collect())
        };
        let list = options::rows()
            .iter()
            .enumerate()
            .filter_map(rows)
            .collect();
        let (rows, pages) = self.grouped(list);
        (VoiceSheet::Options(rows), pages)
    }

    /// The folder list: the drives, or the shown folder's subfolders.
    fn folders_sheet(&mut self, facts: &Facts<'_>) -> (VoiceSheet, Pages) {
        let folder = self
            .browse
            .as_deref()
            .map(|d| Redacted::new(d.to_string_lossy().into_owned()));
        let list = self
            .folder_rows(facts.drives)
            .into_iter()
            .map(|(dir, name)| (Key::Dir(dir), Redacted::new(name)));
        let (rows, pages) = self.paged(list.collect());
        (VoiceSheet::Folders(FolderList { folder, rows }), pages)
    }
}

/// A model's shown name: its row's name, else its file's name; a cloud pick's listed model name, else its typed id.
pub(crate) fn shown_name(name: &Redacted<String>) -> Redacted<String> {
    if let Some((service, model)) = cloud_pick(name.expose()) {
        let row = services::table().ok().and_then(|t| t.get(service));
        let listed = row.and_then(|r| r.model(model)).map(|m| m.name.as_str());
        return Redacted::new(listed.unwrap_or(model).to_owned());
    }
    let row = crate::models::table()
        .ok()
        .and_then(|t| t.get(name.expose()));
    row.map_or_else(|| name.clone(), |r| Redacted::new(r.name.clone()))
}

/// A cloud pick's service, as its row names it; `None` for a model on this PC.
pub(crate) fn service_of(name: &Redacted<String>) -> Option<ServiceName> {
    let (service, _) = cloud_pick(name.expose())?;
    let row = services::table().ok().and_then(|t| t.get(service));
    Some(row.map_or_else(|| ServiceName::Named(service.to_owned()), Service::shown))
}

/// `n` as a count the view holds, or the largest one.
pub(crate) fn short(n: usize) -> u16 {
    u16::try_from(n).unwrap_or(u16::MAX)
}

/// `groups` laid out `per` rows to a page: a group that fits a page but not what is left of this one starts the next.
fn lay_out<T>(groups: Vec<Vec<(Key, T)>>, per: usize) -> Vec<Vec<(Key, T)>> {
    let mut pages: Vec<Vec<(Key, T)>> = Vec::new();
    for group in groups {
        let left = pages.last().map_or(0, |p| per - p.len());
        if group.len() > left && group.len() <= per {
            pages.push(Vec::new());
        }
        for row in group {
            match pages.last_mut() {
                Some(page) if page.len() < per => page.push(row),
                _ => pages.push(vec![row]),
            }
        }
    }
    pages
}

#[cfg(test)]
mod tests;
