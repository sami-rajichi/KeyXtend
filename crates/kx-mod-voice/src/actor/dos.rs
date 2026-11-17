//! The panel's effects, carried out in order: the folder's reads and writes, a setting, the downloads, History's Insert
//! and Copy, and Compare's run. The slow reads go to jobs (`jobs`), History's edits to the store (`keep`).

use std::path::Path;

use kx_module_api::ports::{
    BarHint, FetchAsk, FetchButton, FetchNote, FetchRun, TextAsk, VoiceLevel,
};
use kx_module_api::{Redacted, Zeroizing};

use super::lanes::Owner;
use super::typing::Source;
use super::{Actor, Back, Msg};
use crate::compare::Act;
use crate::flow::{In, suffixed};
use crate::models::table;
use crate::panel::Do;
use crate::{Insert, VoiceSettings};

/// The round a text names until the typer gives it its own.
const UNSENT: u32 = 0;

impl Actor {
    /// Carries out the panel's effects, in order.
    pub(super) fn dos(&mut self, dos: Vec<Do>) {
        for done in dos {
            self.do_one(done);
        }
    }

    fn do_one(&mut self, done: Do) {
        match done {
            Do::Close => {}
            Do::Devices => self.read_devices(),
            Do::Scan => self.scan(),
            Do::Print { file, bytes } => self.print(file, bytes),
            Do::Facts(print, ask) => self.read_facts(print, ask),
            Do::Browse(dir) => self.browse(&dir),
            Do::Make(dir) => self.make(&dir),
            Do::Set(key, value) => self.set(key, value),
            Do::Fetch { row, ask } => self.fetch(row, ask),
            Do::FetchCall { row, button } => self.fetch_call(&row, button),
            Do::Remove(file) => self.remove(&file),
            Do::Reveal(dir) => self.reveal(dir),
            Do::Check(check) => self.check(&check),
            Do::Checked(checked) => {
                self.held.checks.put(checked);
                self.save_checks();
            }
            Do::Insert(ids) => self.insert(&ids),
            Do::Copy(ids) => self.copy_texts(&ids),
            Do::Compare(act) => self.compare(act),
            edit @ (Do::Pin(_) | Do::Unpin(_) | Do::Delete(_) | Do::DeleteAll(_)) => {
                self.edit(edit);
            }
        }
    }

    /// Reads the models folder, when one is chosen, for the panel.
    pub(super) fn scan(&mut self) {
        let Some(dir) = self.panel.folder().dir().map(Path::to_path_buf) else {
            return;
        };
        let scan = self.ports.files.scan(&dir, exts());
        let facts = self.held.facts(self.ports.calendar.now(), &self.flow);
        let dos = self.panel.scanned(scan, &facts);
        self.dos(dos);
    }

    /// Reads `dir`'s subfolders for the folder list.
    fn browse(&mut self, dir: &Path) {
        let scan = self.ports.files.scan(dir, exts());
        self.panel.browsed(dir, scan);
    }

    /// Makes the chosen models folder.
    fn make(&mut self, dir: &Path) {
        let made = self.ports.files.make(dir);
        let dos = self.panel.made(dir, made);
        self.dos(dos);
    }

    /// Writes `[voice]` key `key`, then follows the settings after it; a refused write shows the fault band.
    fn set(&mut self, key: &'static str, value: toml::Value) {
        let Some(next) = self.held.settings.with(key, value.clone()) else {
            return;
        };
        if let Err(error) = self.ports.handle.set(key, value) {
            tracing::warn!(kind = error.kind(), "a voice setting could not be written");
            if !error.holds() {
                self.panel.note = Some(BarHint::Fault);
                return;
            }
        }
        self.follow(next);
    }

    /// Follows `next`: the flow, the panel's folder and checks, and History's rules.
    fn follow(&mut self, next: VoiceSettings) {
        self.held.settings = next.clone();
        self.step(In::Settings(Box::new(next)));
        let facts = self.held.facts(self.ports.calendar.now(), &self.flow);
        let dos = self.panel.follow(&facts);
        self.dos(dos);
        self.follow_rules();
    }

    /// Starts row `row`'s download; its notes come to the inbox, and a refusal goes to the panel as its fault.
    fn fetch(&mut self, row: String, ask: FetchAsk) {
        let (back, id) = (self.ports.back.clone(), row.clone());
        let tell = Box::new(move |note: FetchNote| {
            let row = id.clone();
            drop(back.send(Msg::Back(Back::Fetched { row, note })));
        });
        match self.ports.files.fetch(ask, tell) {
            Ok(run) => drop(self.jobs.fetches.insert(row, run)),
            Err(error) => self.fetched(&row, FetchNote::Fault(error)),
        }
    }

    /// A note from row `row`'s download, for the panel; an ended download is let go.
    pub(super) fn fetched(&mut self, row: &str, note: FetchNote) {
        if matches!(note, FetchNote::Done | FetchNote::Fault(_)) {
            self.jobs.fetches.remove(row);
        }
        let dos = self.panel.fetch_note(row, note);
        self.dos(dos);
    }

    /// Pauses, resumes or cancels row `row`'s download.
    fn fetch_call(&mut self, row: &str, button: FetchButton) {
        let Some(run) = self.jobs.fetches.get_mut(row) else {
            return;
        };
        match button {
            FetchButton::Pause => run.pause(),
            FetchButton::Resume => run.resume(),
            FetchButton::Cancel => run.cancel(),
        }
    }

    /// Deletes a model file; a refusal shows the folder band.
    fn remove(&mut self, file: &Path) {
        if let Err(error) = self.ports.files.remove(file) {
            tracing::warn!(%error, "a model file could not be deleted");
            self.panel.note = Some(BarHint::Folder);
        }
    }

    /// Types History's texts `ids`, in order, joined by the join, the suffix after the last; their answers never reach
    /// the flow. With Paste at once they go as one text, but into a password box.
    fn insert(&mut self, ids: &[u64]) {
        let s = &self.held.settings;
        let (suffix, join, gap_ms) = (&s.type_suffix, &s.history_join, s.type_gap_ms);
        let entries: Vec<_> = (ids.iter().filter_map(|id| self.held.history.get(*id))).collect();
        let last = entries.len().saturating_sub(1);
        let texts: Vec<_> = (entries.iter().enumerate())
            .map(|(at, e)| suffixed(e.text.expose(), if at == last { suffix } else { join }))
            .collect();
        if s.way() == Insert::Paste && !self.flow.password() && !texts.is_empty() {
            let all: Zeroizing<String> =
                Zeroizing::new(texts.iter().map(|t| t.expose().as_str()).collect());
            let (round, erase, text) = (UNSENT, 0, Redacted::new(all));
            return self.paste_text(
                Source::History,
                TextAsk {
                    round,
                    erase,
                    text,
                    gap_ms,
                },
            );
        }
        for text in texts {
            let ask = TextAsk {
                round: UNSENT,
                erase: 0,
                text,
                gap_ms,
            };
            self.type_text(Source::History, ask);
        }
    }

    /// Copies History's texts `ids`, in order, joined by the join; a failure shows the fault band.
    fn copy_texts(&mut self, ids: &[u64]) {
        let texts = ids.iter().filter_map(|id| self.held.history.get(*id));
        let texts: Vec<&str> = texts.map(|entry| entry.text.expose().as_str()).collect();
        let joined = Zeroizing::new(texts.join(&self.held.settings.history_join));
        if !self.clip_text(&joined) {
            self.panel.note = Some(BarHint::Fault);
        }
    }

    /// An effect on Compare's run.
    fn compare(&mut self, act: Act) {
        match act {
            Act::Open { take, ask } => self.open_run((Owner::Compare, take), ask),
            Act::Run { take, call } => self.call_run((Owner::Compare, take), call),
            Act::Level(level) => self.publish(&VoiceLevel { level }),
            Act::Save(board) => {
                if self.held.scores.put(board) {
                    self.save_scores();
                }
            }
            Act::Fell { card, model } => self.fell(&card, &model),
        }
    }
}

/// The model files' extensions the folder lists; none should the compiled-in table fail.
fn exts() -> &'static [String] {
    table().map_or(&[], |t| t.exts())
}

/// A download on the platform's side, kept by its row.
pub(super) type Fetches = std::collections::HashMap<String, Box<dyn FetchRun>>;
