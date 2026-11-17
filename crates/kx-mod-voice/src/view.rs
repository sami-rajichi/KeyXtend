//! The Voice panel's view (design "The gear and the Voice panel"): each page's rows, `page_rows` to a page with the
//! ticks shown, the buttons that act now and the note band. It holds no words; the ui words each value.

mod compare;
mod device;
mod history;
mod models;

use kx_module_api::ports::{
    BarHint, Device, DeviceKind, Drive, FolderList, LayoutChanged, LocalTime, Pages, PanelButton,
    VoiceHome, VoiceLang, VoicePanel, VoiceSheet, VoiceTab,
};
use kx_module_api::{Redacted, is_arabic};

use crate::VoiceSettings;
use crate::counts::{Checks, Scores};
use crate::flow::Supply;
use crate::history::History;
use crate::panel::{Key, Page, Panel, options};

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

impl Panel {
    /// The panel to draw while it is open; it notes each row's meaning, so a click names what it saw.
    pub fn view(&mut self, facts: &Facts<'_>) -> Option<VoicePanel> {
        let page = self.page()?;
        self.sync(facts);
        let (sheet, pages) = match page {
            // The panel never opens the Cloud page until its state is built (P11 part 2).
            Page::Home | Page::Tab(VoiceTab::Cloud) => {
                (VoiceSheet::Home(home(self, facts)), self.single())
            }
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
            Some(Page::Tab(VoiceTab::Options | VoiceTab::Cloud)) | None => {}
        }
        if self.page().is_some_and(|p| p != Page::Home) {
            acts.insert(0, PanelButton::Back);
        }
        acts
    }

    /// The shown page of `list`, `page_rows` to a page, kept within it; notes what each shown row stands for.
    pub(crate) fn paged<T>(&mut self, list: Vec<(Key, T)>) -> (Vec<T>, Pages) {
        let per = self.rows.max(1);
        let count = list.len().div_ceil(per).max(1);
        let at = self.at.min(count - 1);
        self.at = at;
        let (keys, rows): (Vec<Key>, Vec<T>) = list.into_iter().skip(at * per).take(per).unzip();
        self.shown = keys;
        let pages = Pages {
            at: short(at),
            count: short(count),
        };
        (rows, pages)
    }

    /// One page and no rows, as the home list and the set text.
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

    /// The Options page's shown rows.
    fn options_sheet(&mut self, facts: &Facts<'_>) -> (VoiceSheet, Pages) {
        let table = options::values(facts.settings);
        let cores = facts.cpu().map(|d| d.cores);
        let row =
            |(i, o): (usize, &'static options::Opt)| Some((Key::Opt(i), o.row(&table, cores)?));
        let list = options::rows().iter().enumerate().filter_map(row).collect();
        let (rows, pages) = self.paged(list);
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

/// The home list's values for the language in front.
fn home(panel: &Panel, facts: &Facts<'_>) -> VoiceHome {
    let ready = panel.stock(facts).supply(facts.iso()).ok();
    let compared = facts.scores.get(facts.iso()).map(|b| b.day);
    VoiceHome {
        model: ready.as_ref().map(|r| shown_name(&r.name)),
        device: device::label(facts, ready.as_ref()),
        compared,
        history: u32::try_from(facts.history.len()).unwrap_or(u32::MAX),
        changed: options::changed(facts.settings),
    }
}

/// A model's shown name: its row's name, else its file's name.
pub(crate) fn shown_name(name: &Redacted<String>) -> Redacted<String> {
    let row = crate::models::table()
        .ok()
        .and_then(|t| t.get(name.expose()));
    row.map_or_else(|| name.clone(), |r| Redacted::new(r.name.clone()))
}

/// `n` as a count the view holds, or the largest one.
pub(crate) fn short(n: usize) -> u16 {
    u16::try_from(n).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests;
