//! The Models page's acts: Use for this language, Download with the first download's folder question (edge 38), Delete
//! file, a download's Pause, Resume and Cancel, and the folder's reads, fingerprints and facts coming back.

use std::path::{Path, PathBuf};

use kx_module_api::ports::{
    BarHint, FetchButton, FetchNote, Fingerprint, FolderScan, ModelError, ModelFacts, ModelState,
    PanelButton, SpeechError, VoiceTab, path_ok,
};

use super::{Choice, Do, Key, Page, Panel};
use crate::fetch::{Fetched, hint};
use crate::folder::File;
use crate::models::table;
use crate::settings::{MODELS_DIR, PICKS};
use crate::view::Facts;

impl Panel {
    /// A Models page or folder list button.
    pub(super) fn models_button(&mut self, button: PanelButton, facts: &Facts<'_>) -> Vec<Do> {
        let ticked = match self.ticked() {
            Some(Key::Model(choice)) => Some(choice.clone()),
            _ => None,
        };
        match (button, ticked) {
            (PanelButton::Use, Some(choice)) => self.pick(&choice, facts),
            (PanelButton::Download, Some(Choice::Row(id))) => self.download(&id, facts),
            (PanelButton::Delete, Some(choice)) => self.delete_file(&choice),
            (PanelButton::Change, _) => self.go(Page::Folders),
            (PanelButton::Open, _) => self
                .folder
                .dir()
                .map(|d| Do::Reveal(d.to_path_buf()))
                .into_iter()
                .collect(),
            (PanelButton::Here, _) => self
                .browse
                .clone()
                .map_or_else(Vec::new, |d| self.choose(d)),
            _ => Vec::new(),
        }
    }

    /// Use `choice` for the language in front: its pick is written, by row id or file name.
    pub(super) fn pick(&mut self, choice: &Choice, facts: &Facts<'_>) -> Vec<Do> {
        let value = match choice {
            Choice::Row(id) | Choice::File(id) => id.clone(),
        };
        let mut picks = facts.settings.picks.clone();
        picks.insert(facts.iso().to_owned(), value);
        self.ticks.clear();
        toml::Value::try_from(&picks)
            .map(|v| Do::Set(PICKS, v))
            .into_iter()
            .collect()
    }

    /// Download language `iso`'s model (`Out::Download`): the Models page opens and the pick, else the default, starts.
    pub fn download_for(&mut self, iso: &str, facts: &Facts<'_>) -> Vec<Do> {
        let mut out = Vec::new();
        if self.page != Some(Page::Tab(VoiceTab::Models)) {
            out = self.go(Page::Tab(VoiceTab::Models));
        }
        if let Some(row) = self.stock(facts).wanted(iso) {
            let id = row.id.clone();
            out.extend(self.download(&id, facts));
        }
        out
    }

    /// Starts row `id`'s download; the first one asks for the folder, one waits while the folder is read, and a lack of
    /// room shows its note (edge 21). A name another file holds ticks the row, so Delete file is at hand.
    pub(super) fn download(&mut self, id: &str, facts: &Facts<'_>) -> Vec<Do> {
        let Some(row) = table().ok().and_then(|t| t.get(id)) else {
            return Vec::new();
        };
        if self.fetches.busy() {
            return Vec::new();
        }
        if self.folder.dir().is_none() {
            match crate::folder::offer(facts.drives) {
                Some(_) => self.offer = Some(id.to_owned()),
                None => self.note = Some(BarHint::Folder),
            }
            return Vec::new();
        }
        if self.folder.reading() {
            self.waiting = Some(id.to_owned());
            return Vec::new();
        }
        match self
            .fetches
            .start(row, &self.folder, facts.drives, facts.settings)
        {
            Ok(ask) => vec![Do::Fetch {
                row: id.to_owned(),
                ask,
            }],
            Err(note) => {
                if self.folder.taken(row).is_some() {
                    self.ticks = vec![Key::Model(Choice::Row(id.to_owned()))];
                }
                self.note = Some(note);
                Vec::new()
            }
        }
    }

    /// The folder is read: a waiting download starts if its model is still missing, and a picked card gets the check
    /// a new file needs (edge 26).
    fn settled(&mut self, facts: &Facts<'_>) -> Vec<Do> {
        if self.folder.reading() {
            return Vec::new();
        }
        let mut out = self.recheck(facts);
        let waiting = self.waiting.take();
        let missing = |id: &String| {
            let state = self.state_of(&Choice::Row(id.clone()), facts);
            state == Some(ModelState::Missing)
        };
        if let Some(id) = waiting.filter(missing) {
            out.extend(self.download(&id, facts));
        }
        out
    }

    /// The file `choice` stands for: an other file, a row's own, or the one holding a row's name.
    fn file_of(&self, choice: &Choice) -> Option<&File> {
        match choice {
            Choice::Row(id) => {
                let row = table().ok()?.get(id)?;
                self.folder.of(row).or_else(|| self.folder.taken(row))
            }
            Choice::File(name) => self.folder.file(name),
        }
    }

    /// The file holding row `choice`'s name that is not its own, which waits for Delete file before Download.
    pub(crate) fn taken(&self, choice: &Choice) -> Option<&File> {
        let Choice::Row(id) = choice else {
            return None;
        };
        self.folder.taken(table().ok()?.get(id)?)
    }

    /// Deletes `choice`'s file, then reads the folder again.
    fn delete_file(&mut self, choice: &Choice) -> Vec<Do> {
        let name = self.file_of(choice);
        let file = name.zip(self.folder.dir()).map(|(f, d)| d.join(&f.name));
        self.ticks.clear();
        file.map_or_else(Vec::new, |f| vec![Do::Remove(f), Do::Scan])
    }

    /// Uses `dir` as the models folder once it is made; a relative path, a device or a climb is refused (edge 38).
    pub(super) fn choose(&mut self, dir: PathBuf) -> Vec<Do> {
        if !path_ok(&dir) {
            self.note = Some(BarHint::Folder);
            return Vec::new();
        }
        self.making = Some(dir.clone());
        vec![Do::Make(dir)]
    }

    /// The chosen folder was made, or not: it becomes `models_dir` and is read, then a waiting first download starts
    /// unless its model is already there.
    pub fn made(&mut self, dir: &Path, made: Result<(), ModelError>) -> Vec<Do> {
        if self.making.as_deref() != Some(dir) {
            return Vec::new();
        }
        self.making = None;
        if let Err(error) = made {
            self.note = hint(error);
            self.offer = None;
            return Vec::new();
        }
        self.folder.set_dir(dir);
        if self.page == Some(Page::Folders) {
            self.go(Page::Tab(VoiceTab::Models));
        }
        self.waiting = self.offer.take();
        let path = toml::Value::String(dir.to_string_lossy().into_owned());
        vec![Do::Set(MODELS_DIR, path), Do::Scan]
    }

    /// A download's button on the shown page's row `at`.
    pub(super) fn fetch_button(&mut self, at: usize, button: FetchButton) -> Vec<Do> {
        let Some(Key::Model(Choice::Row(id))) = self.shown.get(at).cloned() else {
            return Vec::new();
        };
        let pressed = self.fetches.press(&id, button);
        pressed
            .map(|button| Do::FetchCall { row: id, button })
            .into_iter()
            .collect()
    }

    /// A note from row `id`'s download: a checked file is known at once, a fault shows its note.
    pub fn fetch_note(&mut self, id: &str, note: FetchNote) -> Vec<Do> {
        match self.fetches.note(id, note) {
            Fetched::Done => {
                if let Some(row) = table().ok().and_then(|t| t.get(id)) {
                    self.folder.fetched(row);
                }
                vec![Do::Scan]
            }
            Fetched::Failed(note) => {
                self.note = Some(note);
                Vec::new()
            }
            Fetched::Moved | Fetched::Stopped | Fetched::Stale => Vec::new(),
        }
    }

    /// The models folder as read: each file not read yet gets its fingerprint read, else the folder is settled.
    pub fn scanned(&mut self, scan: Result<FolderScan, ModelError>, facts: &Facts<'_>) -> Vec<Do> {
        let unread = self.folder.scanned(scan).into_iter();
        let unread: Vec<Do> = unread
            .map(|(file, bytes)| Do::Print { file, bytes })
            .collect();
        if unread.is_empty() {
            return self.settled(facts);
        }
        unread
    }

    /// The fingerprint of `file` of `bytes`, as `Do::Print` asked: an other file gets its facts read, in a job capped
    /// at its size plus `job_mb`; the last one settles the folder.
    pub fn printed(
        &mut self,
        file: &Path,
        bytes: u64,
        print: Result<Fingerprint, ModelError>,
        facts: &Facts<'_>,
    ) -> Vec<Do> {
        let known = print.as_ref().ok().copied();
        let other = self.folder.printed(file, bytes, print);
        let job = facts.settings.job();
        let ask = (other.zip(self.folder.dir())).and_then(|(o, dir)| o.facts(dir, job));
        let mut out: Vec<Do> = (known.zip(ask))
            .map(|(p, a)| Do::Facts(p, a))
            .into_iter()
            .collect();
        out.extend(self.settled(facts));
        out
    }

    /// An other file's facts, or the error that left them unread.
    pub fn facts_read(&mut self, print: Fingerprint, facts: Result<ModelFacts, SpeechError>) {
        self.folder.facts_read(print, facts);
    }
}
