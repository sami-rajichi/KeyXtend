//! What the actor works through: the bus, the clocks, every service the manifest names, its own settings, its inbox,
//! how a job starts, and the videos a delete missed, kept for the module's next start.

use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use kx_module_api::ports::{
    Calendar, ClipboardWriter, FileAsk, FolderReveal, MediaOpen, MicPasses, Pictures, Recorder,
    ScreenFreeze, SnipFile, SnipFolder,
};
use kx_module_api::{Bus, Clock, SettingsHandle};

use super::Msg;

/// One piece of work run apart from the actor.
pub(crate) type Job = Box<dyn FnOnce() + Send>;

/// Starts a job on its own thread, or says false when it cannot; tests run it in place.
pub(crate) type Spawn = fn(Job) -> bool;

/// The unsaved part's files the module tracks for as long as the app runs, so an app restart relies on the sweep
/// deleting every video not marked copied.
#[derive(Debug, Default)]
pub(crate) struct Unsaved {
    /// The deletes the folder refused, which each sweep tries again.
    pub missed: Vec<SnipFile>,
    /// The files a job still uses, once per use, which no sweep deletes.
    pub used: Vec<SnipFile>,
}

/// The unsaved part's tracked files, shared by every start of the module and its jobs.
pub(crate) type Leftovers = Arc<Mutex<Unsaved>>;

/// The leftovers, locked; a panic elsewhere leaves them as they were.
pub(super) fn held(leftovers: &Leftovers) -> MutexGuard<'_, Unsaved> {
    leftovers.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A file a job uses, which no sweep deletes until the job drops it.
pub(super) struct Using {
    leftovers: Leftovers,
    file: SnipFile,
}

impl Using {
    /// Marks `file` used until the guard drops.
    pub(super) fn new(leftovers: &Leftovers, file: SnipFile) -> Self {
        held(leftovers).used.push(file.clone());
        Self {
            leftovers: Arc::clone(leftovers),
            file,
        }
    }
}

impl Drop for Using {
    fn drop(&mut self) {
        let mut held = held(&self.leftovers);
        if let Some(at) = held.used.iter().position(|f| *f == self.file) {
            held.used.swap_remove(at);
        }
    }
}

/// The actor's ports, from the module's start or a test.
pub(crate) struct Ports {
    /// The bus, limited to the module's grants.
    pub bus: Bus,
    /// Monotonic time, for the countdown, the ticks, the notes and the clipboard's tries.
    pub clock: Arc<dyn Clock>,
    /// The wall time, which names a snip in the Save dialog.
    pub calendar: Arc<dyn Calendar>,
    /// Freezes the screens.
    pub freeze: Arc<dyn ScreenFreeze>,
    /// Records parts and writes re-timed copies.
    pub recorder: Arc<dyn Recorder>,
    /// The app's microphone passes.
    pub mics: Arc<dyn MicPasses>,
    /// The Snips folder and its unsaved part.
    pub folder: Arc<dyn SnipFolder>,
    /// The system's Save and folder dialogs.
    pub dialogs: Arc<dyn FileAsk>,
    /// Plays a video in the user's own player.
    pub player: Arc<dyn MediaOpen>,
    /// Copy, kept by the clipboard histories.
    pub clip: Arc<dyn ClipboardWriter>,
    /// Makes a picture's PNG in the media worker.
    pub pictures: Arc<dyn Pictures>,
    /// Open folder.
    pub reveal: Arc<dyn FolderReveal>,
    /// The module's own settings, for the panel's and the editor's writes.
    pub handle: SettingsHandle,
    /// The actor's own inbox, where each job and recording sends its answer.
    pub back: Sender<Msg>,
    /// Starts each job off the actor.
    pub spawn: Spawn,
    /// The videos a delete missed.
    pub leftovers: Leftovers,
}
