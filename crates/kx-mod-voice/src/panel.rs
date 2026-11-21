//! The Voice panel's state (design "The gear and the Voice panel"): the page, its paging by the ui's rows, the rows
//! ticked, the questions and the note band, beside the models folder, the downloads and Compare. A click changes it
//! and says what the actor does next, as `Do`s; `view` draws it with the facts the actor holds.

mod act;
mod cloud;
mod device;
mod folders;
mod history;
mod lines;
mod models;
pub mod options;
mod pages;
mod rows;
mod sides;
mod trial;
mod typing;

pub use act::{Check, Do};
pub(crate) use cloud::Clouds;

use std::path::PathBuf;

use kx_module_api::Redacted;
use kx_module_api::ports::{
    BarButton, BarHint, CardId, DevicePick, Fingerprint, PanelButton, VoiceAct, VoiceClick,
    VoicePage, VoiceTab,
};

use crate::VoiceSettings;
use crate::compare::{Act, Compare};
use crate::fetch::Fetches;
use crate::folder::{Folder, Stock};
use crate::history::Asked;
use crate::view::Facts;

/// A page of the panel.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Page {
    /// The home list.
    Home,
    /// A page the home list opens.
    Tab(VoiceTab),
    /// The folder list, to pick the models folder.
    Folders,
    /// The shown service's card.
    Card,
    /// The shown service's key page.
    Lines,
}

/// A model on the Models page: a row of the table by its id, or an other file by its name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Choice {
    /// A row of the table.
    Row(String),
    /// A file no row names.
    File(String),
}

/// What a shown row stands for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Key {
    Model(Choice),
    Device(DevicePick),
    Score(Fingerprint),
    Text(u64),
    Opt(usize),
    Dir(PathBuf),
    /// A service on the Cloud page, by its id.
    Service(String),
    /// A cloud model, by its service's id and the id a pick names, redacted as a typed Other id is the user's.
    Cloud {
        service: String,
        model: Redacted<String>,
    },
    /// A row a click does nothing on, as a model's turn while Compare runs.
    Plain,
}

/// The panel's state; it is closed until `open`.
#[derive(Clone, Debug, Default)]
pub struct Panel {
    pub(crate) page: Option<Page>,
    /// Rows a page shows, the ui's `page_rows` once it tells them; 0 counts as 1.
    pub(crate) rows: usize,
    /// The shown page of the list, or the one holding `hold`.
    pub(crate) at: usize,
    /// The option row a stepper or chip click keeps on screen until the page turns.
    pub(crate) hold: Option<Key>,
    pub(crate) ticks: Vec<Key>,
    /// What each row of the last view stands for.
    pub(crate) shown: Vec<Key>,
    pub(crate) note: Option<BarHint>,
    pub(crate) asked: Option<Asked>,
    /// The page and rows the flow last drew, and the rounds since they last changed and of the latest view, so a row
    /// click names the rows it saw; a tick or a download's progress keeps them.
    pub(crate) face: (Option<Page>, Vec<Key>),
    pub(crate) rounds: (u32, u32),
    /// The row a first download waits on while the folder question shows.
    pub(crate) offer: Option<String>,
    /// The row a download waits on while the folder is read, as a copy may be there under another name.
    pub(crate) waiting: Option<String>,
    /// The folder being made.
    pub(crate) making: Option<PathBuf>,
    /// The folder list's folder; `None` at the drives.
    pub(crate) browse: Option<PathBuf>,
    /// The folder list folder's subfolders.
    pub(crate) dirs: Vec<String>,
    /// The card being checked and its warm-up's percent.
    pub(crate) checking: Option<(CardId, u8)>,
    /// `recheck` waits for the running check, dictation or Compare to end.
    pub(crate) queued: bool,
    pub(crate) folder: Folder,
    pub(crate) fetches: Fetches,
    pub(crate) compare: Compare,
    pub(crate) cloud: Clouds,
}

impl Panel {
    /// A closed panel over the folder `settings` name.
    #[must_use]
    pub fn new(settings: &VoiceSettings) -> Self {
        Self {
            folder: Folder::new(&settings.models_dir),
            ..Self::default()
        }
    }

    /// Opens it on the home list; the reads its page needs follow.
    pub fn open(&mut self) -> Vec<Do> {
        if self.page.is_some() {
            return Vec::new();
        }
        self.go(Page::Home)
    }

    /// The page shown, while open.
    #[must_use]
    pub fn page(&self) -> Option<Page> {
        self.page
    }

    /// The models folder.
    #[must_use]
    pub fn folder(&self) -> &Folder {
        &self.folder
    }

    /// The downloads.
    #[must_use]
    pub fn fetches(&self) -> &Fetches {
        &self.fetches
    }

    /// Compare, whose recording the bar shows while it runs.
    #[must_use]
    pub fn compare(&self) -> &Compare {
        &self.compare
    }

    /// What serves each language, from the folder and `facts`; the flow's `Supply`.
    #[must_use]
    pub fn stock<'a>(&'a self, facts: &Facts<'a>) -> Stock<'a> {
        Stock {
            settings: facts.settings,
            folder: &self.folder,
            devices: facts.devices,
            checks: facts.checks,
            clips: facts.clips,
        }
    }

    /// Notes a page the flow told (each `Out::View`, after `view`), so a row click names the rows it saw.
    pub fn told(&mut self, page: &VoicePage) {
        let face = (
            self.page.filter(|_| page.panel.is_some()),
            self.shown.clone(),
        );
        if self.face != face {
            self.face = face;
            self.rounds.0 = page.round;
        }
        self.rounds.1 = page.round;
    }

    /// True for a click on a row of a view whose rows changed since: its round lies outside the drawn rows' last change
    /// and the latest view, wrapping.
    fn stale(&self, act: &VoiceAct) -> bool {
        let at_row = matches!(
            act.click,
            VoiceClick::Row(_)
                | VoiceClick::Fetch { .. }
                | VoiceClick::Flip(_)
                | VoiceClick::Step { .. }
        );
        let (since, latest) = self.rounds;
        at_row && latest.wrapping_sub(act.round) > latest.wrapping_sub(since)
    }

    /// A click on the panel, or the bar's Stop or Cancel that the flow gave Compare (`Out::Lent`). A click on a row of
    /// a view whose rows changed since is dropped.
    pub fn click(&mut self, act: &VoiceAct, facts: &Facts<'_>) -> Vec<Do> {
        let row = |r: &u8| usize::from(*r);
        if self.stale(act) {
            return Vec::new();
        }
        match &act.click {
            VoiceClick::Rows(rows) => self.rows = row(rows),
            VoiceClick::Open(VoiceTab::Cloud)
            | VoiceClick::Cloud(_)
            | VoiceClick::Service(_)
            | VoiceClick::SetKey(_)
            | VoiceClick::SetAddress(_)
            | VoiceClick::SetModel(_)
            | VoiceClick::Save(_)
            | VoiceClick::Paste(_)
            | VoiceClick::Caret { .. }
            | VoiceClick::KeepNothing(_)
            | VoiceClick::Side(_) => return self.cloud_click(&act.click, facts),
            VoiceClick::Open(tab) if self.page == Some(Page::Home) => {
                return self.go(Page::Tab(*tab));
            }
            VoiceClick::Page { back } => self.turn(*back),
            VoiceClick::Row(at) => return self.row(row(at)),
            VoiceClick::Fetch { row: at, button } => return self.fetch_button(row(at), *button),
            VoiceClick::Flip(at) => return self.flip(row(at), facts.settings),
            VoiceClick::Step { row: at, up } => return self.step(row(at), *up, facts.settings),
            VoiceClick::Button(button) => return self.button(*button, facts),
            VoiceClick::Compare(set) => return self.start(set, facts),
            VoiceClick::Confirm(yes) => return self.confirm(*yes, facts),
            VoiceClick::Bar(BarButton::Stop) => return wrap(self.compare.stop()),
            VoiceClick::Bar(BarButton::Cancel) => return wrap(self.compare.cancel()),
            // "Use the local model" is the flow's.
            VoiceClick::Open(_) | VoiceClick::Bar(_) | VoiceClick::Over(_) | VoiceClick::Local => {}
        }
        Vec::new()
    }

    /// Follows `[voice]` after a change: a new models folder is read, and a picked card gets the check it lacks.
    pub fn follow(&mut self, facts: &Facts<'_>) -> Vec<Do> {
        if self.folder.follow(&facts.settings.models_dir) {
            self.waiting = None;
            return vec![Do::Scan];
        }
        self.recheck(facts)
    }

    /// The model file was not found (`Out::Lost`): the folder is read again.
    #[must_use]
    pub fn lost(&self) -> Vec<Do> {
        vec![Do::Scan]
    }

    /// Closes the panel, stopping Compare; a Paste or key test waited for is dropped.
    pub fn close(&mut self) -> Vec<Do> {
        let mut out = wrap(self.compare.cancel());
        self.compare.reset();
        (self.page, self.offer, self.asked) = (None, None, None);
        self.cloud.reset();
        out.push(Do::Close);
        out
    }

    /// A header or footer button, where the page offers it now.
    fn button(&mut self, button: PanelButton, facts: &Facts<'_>) -> Vec<Do> {
        if !self.acts(facts).contains(&button) {
            return Vec::new();
        }
        self.note = None;
        if let (PanelButton::Use, Some((service, model))) = (button, self.cloud_ticked()) {
            return self.cloud_pick(&service, model.expose(), facts);
        }
        match (button, self.page) {
            (PanelButton::Back, _) => self.back(),
            (PanelButton::Close, _) => self.close(),
            (_, Some(Page::Tab(VoiceTab::Models) | Page::Folders)) => {
                self.models_button(button, facts)
            }
            (_, Some(Page::Tab(VoiceTab::Device))) => self.device_button(button, facts),
            (_, Some(Page::Tab(VoiceTab::Compare))) => self.compare_button(button, facts),
            (_, Some(Page::Tab(VoiceTab::History))) => self.history_button(button, facts),
            _ => Vec::new(),
        }
    }
}

/// Compare's effects as the actor's.
fn wrap(acts: Vec<Act>) -> Vec<Do> {
    acts.into_iter().map(Do::Compare).collect()
}

#[cfg(test)]
pub(crate) mod rig;
#[cfg(test)]
mod tests;
