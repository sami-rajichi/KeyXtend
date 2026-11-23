//! A speech library the test scripts: its devices, its languages, which loads and runs fail and when, and what it was
//! asked, so the engine's rules run without a model file or a card.

use crate::voice::library::{Failed, Found, Library, Loaded, Place, Stop};
use kx_ipc::DeviceKind;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use zeroize::Zeroizing;

/// What the fake library does, and what it saw.
#[derive(Default)]
pub struct Script {
    /// No processor part loaded, so it does not start.
    pub broken: bool,
    /// The devices it finds.
    pub found: Vec<Found>,
    /// The languages its models know.
    pub langs: Vec<String>,
    /// Its models' longest run, in samples.
    pub most: Option<usize>,
    /// How a load on a card or on the processor fails, if it does.
    pub card_load: Option<Failed>,
    pub cpu_load: Option<Failed>,
    /// Card runs that succeed before each later one fails with `Device`.
    pub card_runs: Option<usize>,
    /// The text a card run gives in place of its length and language, as a card that computes wrongly would.
    pub card_text: Option<String>,
    /// A load panics, as a broken library might.
    pub panic: bool,
    /// The next load waits for one message here, as a slow first load does; a test sends it, so nothing sleeps.
    pub gate: Option<Receiver<()>>,
    /// Its starts, loads with their threads, and runs with their length and language.
    pub starts: usize,
    pub loads: Vec<(Place, usize)>,
    pub runs: Vec<(Place, usize, String)>,
    /// The files whose languages were read.
    pub read: Vec<PathBuf>,
}

/// The fake library over a shared script.
#[derive(Clone, Default)]
pub struct FakeLib(pub Arc<Mutex<Script>>);

impl FakeLib {
    /// The script, to set or read.
    pub fn script(&self) -> MutexGuard<'_, Script> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// A model of the fake library: each run's text is its length and language.
pub struct FakeModel {
    lib: FakeLib,
    place: Place,
    langs: Vec<String>,
    most: Option<usize>,
}

impl Library for FakeLib {
    const RUNNER: &'static str = "fake";
    type Model = FakeModel;

    fn start(&self) -> bool {
        let mut s = self.script();
        s.starts += 1;
        !s.broken
    }

    fn devices(&self) -> Vec<Found> {
        self.script().found.clone()
    }

    fn langs(&self, path: &Path) -> Result<Vec<String>, Failed> {
        let mut s = self.script();
        s.read.push(path.to_path_buf());
        s.cpu_load.map_or_else(|| Ok(s.langs.clone()), Err)
    }

    fn load(&self, _: &Path, place: &Place, threads: usize) -> Result<FakeModel, Failed> {
        let gate = self.script().gate.take();
        if let Some(gate) = gate {
            let _ = gate.recv();
        }
        let mut s = self.script();
        s.loads.push((place.clone(), threads));
        assert!(!s.panic, "the fake library panics");
        let fails = if place.card() {
            s.card_load
        } else {
            s.cpu_load
        };
        fails.map_or(Ok(()), Err)?;
        Ok(FakeModel {
            lib: self.clone(),
            place: place.clone(),
            langs: s.langs.clone(),
            most: s.most,
        })
    }
}

impl Loaded for FakeModel {
    fn langs(&self) -> &[String] {
        &self.langs
    }

    fn most(&self) -> Option<usize> {
        self.most
    }

    fn run(&mut self, pcm: &[f32], lang: &str, stop: &Stop) -> Result<Zeroizing<String>, Failed> {
        if stop.is_set() {
            return Err(Failed::Stopped);
        }
        let mut s = self.lib.script();
        s.runs
            .push((self.place.clone(), pcm.len(), lang.to_owned()));
        if self.place.card() {
            let ok = s.runs.iter().filter(|r| r.0.card()).count();
            if s.card_runs.is_some_and(|n| ok > n) {
                return Err(Failed::Device);
            }
            if let Some(text) = &s.card_text {
                return Ok(Zeroizing::new(text.clone()));
            }
        }
        Ok(Zeroizing::new(format!("{} {lang}", pcm.len())))
    }
}

/// A device of `kind` named `id`, with `free_mb` of 8192 MB free.
pub fn device(kind: DeviceKind, id: &str, free_mb: u64) -> Found {
    let mb = 1 << 20;
    Found {
        kind,
        id: id.into(),
        name: format!("{id} name"),
        total: 8192 * mb,
        free: free_mb * mb,
    }
}
