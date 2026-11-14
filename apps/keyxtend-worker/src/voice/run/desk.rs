//! The engine thread's state: the library, the model it holds and the run it writes. A run's model is placed by the
//! fit check; a card that fails to load, or fails on a piece, hands the same sound to the processor with a note.

mod write;

use super::Cmd;
use crate::voice::Tell;
use crate::voice::cards;
use crate::voice::check;
use crate::voice::library::{Failed, Library, Loaded, Place, Stop, ms_since};
use kx_ipc::{Model, Stage, Target, VoiceFault, VoiceNote};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// A run's start, as the job's thread hands it over.
pub struct Order {
    /// The run.
    pub id: u32,
    /// Its model.
    pub model: Model<'static>,
    /// Where its notes go.
    pub tell: Tell,
    /// Set when its writer drops.
    pub stop: Stop,
}

/// The model held between runs, and where and how it was loaded.
struct Held<M> {
    path: PathBuf,
    place: Place,
    threads: usize,
    model: M,
}

/// The run being written.
struct Writing {
    id: u32,
    lang: String,
    tell: Tell,
    stop: Stop,
    told: u16,
    card: bool,
    load_ms: u32,
}

/// The engine thread's state.
pub struct Desk<L: Library> {
    lib: L,
    cores: usize,
    started: Option<bool>,
    held: Option<Held<L::Model>>,
    run: Option<Writing>,
}

impl<L: Library> Desk<L> {
    /// A desk over `lib` on `cores` real cores, its parts not loaded yet.
    pub fn new(lib: L, cores: usize) -> Self {
        Self {
            lib,
            cores,
            started: None,
            held: None,
            run: None,
        }
    }

    /// Takes one command; an ask that needs the library fails `Runner` when its parts did not load.
    pub fn take(&mut self, cmd: Cmd) {
        match cmd {
            Cmd::Begin(order) => self.begin(&order),
            Cmd::Piece(stop, sound) => self.piece(&stop, &sound),
            Cmd::End(stop, heard_ms, at) => self.end(&stop, heard_ms, at),
            Cmd::Devices(id, tell) => tell.note(self.devices(id)),
            Cmd::Facts(id, path, tell) => tell.note(self.facts(id, &path)),
            Cmd::Check(exam) => {
                let note = self.check(&exam);
                exam.tell.note(note);
            }
        }
    }

    /// The ask a command answers, so a panic inside can be told on it.
    pub fn who(&self, cmd: &Cmd) -> Option<(u32, Tell)> {
        match cmd {
            Cmd::Begin(order) => Some((order.id, order.tell.clone())),
            Cmd::Piece(stop, _) | Cmd::End(stop, ..) => {
                self.mine(stop).map(|r| (r.id, r.tell.clone()))
            }
            Cmd::Devices(id, tell) | Cmd::Facts(id, _, tell) => Some((*id, tell.clone())),
            Cmd::Check(exam) => Some((exam.id, exam.tell.clone())),
        }
    }

    /// Drops the run and the model, after a panic.
    pub fn reset(&mut self) {
        (self.run, self.held) = (None, None);
    }

    /// True once the library's parts loaded; they load on the first ask.
    fn ready(&mut self) -> bool {
        *self.started.get_or_insert_with(|| self.lib.start())
    }

    /// The run `stop` belongs to, while it is written.
    fn mine(&self, stop: &Stop) -> Option<&Writing> {
        self.run.as_ref().filter(|r| r.stop.same(stop))
    }

    fn devices(&mut self, id: u32) -> VoiceNote {
        if !self.ready() {
            return fault(id, VoiceFault::Runner);
        }
        let list = cards::list(&self.lib.devices());
        if list.is_empty() {
            return fault(id, VoiceFault::Internal);
        }
        VoiceNote::Devices { id, list }
    }

    fn facts(&mut self, id: u32, path: &Path) -> VoiceNote {
        if !self.ready() {
            return fault(id, VoiceFault::Runner);
        }
        check::facts(&self.lib, id, path)
    }

    /// A check runs alone: refused while a run is written, and the held model is dropped to make room.
    fn check(&mut self, exam: &check::Exam) -> VoiceNote {
        if !self.ready() {
            return fault(exam.id, VoiceFault::Runner);
        }
        if self.run.is_some() {
            return fault(exam.id, VoiceFault::OutOfStep);
        }
        self.held = None;
        check::check(&self.lib, self.cores, exam)
    }

    /// Starts writing `order`'s run, telling its fault when its model cannot serve.
    fn begin(&mut self, order: &Order) {
        self.run = None;
        if order.stop.is_set() {
            return;
        }
        match self.open(order) {
            Ok(run) => self.run = Some(run),
            Err(why) => order.tell.note(fault(order.id, why)),
        }
    }

    /// The run of `order` once its model is placed, loaded and knows its language.
    fn open(&mut self, order: &Order) -> Result<Writing, VoiceFault> {
        if !self.ready() {
            return Err(VoiceFault::Runner);
        }
        let (id, model) = (order.id, &order.model);
        let path = PathBuf::from(&*model.path);
        let threads = cards::threads(model.threads, self.cores);
        let (place, refused) = self.aim(&path, &model.target);
        if let Some(why) = refused {
            order.tell.note(fault(id, why));
        }
        let (card, load_ms) = self.mount(id, &order.tell, path, place, threads)?;
        let held = self.held.as_ref().ok_or(VoiceFault::Internal)?;
        if !held.model.knows(&model.lang) {
            return Err(VoiceFault::Language);
        }
        let (lang, tell, stop) = (
            model.lang.to_string(),
            order.tell.clone(),
            order.stop.clone(),
        );
        let told = 0;
        Ok(Writing {
            id,
            lang,
            tell,
            stop,
            told,
            card,
            load_ms,
        })
    }

    /// Where `target` runs: a card already holding this file needs no fit check, as its own memory is in use.
    fn aim(&self, path: &Path, target: &Target<'_>) -> (Place, Option<VoiceFault>) {
        let Target::Card { id, .. } = target else {
            return (Place::Cpu, None);
        };
        let here = Place::Card(id.to_string());
        if self
            .held
            .as_ref()
            .is_some_and(|h| h.path == path && h.place == here)
        {
            return (here, None);
        }
        cards::place(target, &self.lib.devices())
    }

    /// Holds the file loaded at `place`, telling `Loading` unless it is held already; true when on a card, and the
    /// load's time.
    fn mount(
        &mut self,
        id: u32,
        tell: &Tell,
        path: PathBuf,
        place: Place,
        threads: usize,
    ) -> Result<(bool, u32), VoiceFault> {
        let same = |h: &Held<L::Model>| h.path == path && h.place == place && h.threads == threads;
        if self.held.as_ref().is_some_and(same) {
            return Ok((place.card(), 0));
        }
        self.held = None;
        tell.note(VoiceNote::Stage {
            id,
            stage: Stage::Loading,
        });
        let start = Instant::now();
        let loaded = match self.lib.load(&path, &place, threads) {
            Err(why) if place.card() && why.card() => {
                tell.note(fault(id, VoiceFault::CardFailed));
                self.lib
                    .load(&path, &Place::Cpu, threads)
                    .map(|m| (m, Place::Cpu))
            }
            other => other.map(|m| (m, place)),
        };
        let (model, place) = loaded.map_err(Failed::fault)?;
        let card = place.card();
        self.held = Some(Held {
            path,
            place,
            threads,
            model,
        });
        Ok((card, ms_since(start)))
    }
}

/// A fault note on `id`.
fn fault(id: u32, fault: VoiceFault) -> VoiceNote {
    VoiceNote::Fault { id, fault }
}
