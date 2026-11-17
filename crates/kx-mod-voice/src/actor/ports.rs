//! What the actor works through: the bus, the clocks, every service the manifest gates, the watches, its own settings,
//! its inbox, how a job starts and the typer.

use std::sync::Arc;
use std::sync::mpsc::Sender;

use kx_module_api::ports::{
    Calendar, ClipboardWriter, CloudSpeech, FieldRun, FolderReveal, ModelFiles, SealedShelf,
    SessionRun, Speech,
};
use kx_module_api::{Bus, Clock, SettingsHandle};

use super::{Msg, Typist};

/// One piece of work run apart from the actor.
pub(crate) type Job = Box<dyn FnOnce() + Send>;

/// Starts a job on its own thread, or says false when it cannot; tests run it in place.
pub(crate) type Spawn = fn(Job) -> bool;

/// The actor's ports, from the module's start or a test.
pub(crate) struct Ports {
    /// The bus, limited to the module's grants.
    pub bus: Bus,
    /// Monotonic time, for the flow's hints and fades.
    pub clock: Arc<dyn Clock>,
    /// The wall time, for History's stamps and the counts' days.
    pub calendar: Arc<dyn Calendar>,
    /// The microphone, the devices, a file's facts and a card's check.
    pub speech: Arc<dyn Speech>,
    /// The drives, the models folder and the downloads.
    pub files: Arc<dyn ModelFiles>,
    /// Copy.
    pub clip: Arc<dyn ClipboardWriter>,
    /// Shows the models folder.
    pub reveal: Arc<dyn FolderReveal>,
    /// The voice store's own shelf, which holds the cloud keys too.
    pub shelf: Arc<dyn SealedShelf>,
    /// Cloud speech, when the module may reach it; without it an upload ends with the cloud fault note.
    pub cloud: Option<Arc<dyn CloudSpeech>>,
    /// The focused box's watch, dropped at the stop.
    pub field: Option<Box<dyn FieldRun>>,
    /// The session's watch, dropped at the stop.
    pub session: Option<Box<dyn SessionRun>>,
    /// The module's own settings, for the panel's writes.
    pub handle: SettingsHandle,
    /// The actor's own inbox, where each job sends its answer.
    pub back: Sender<Msg>,
    /// Starts each job off the actor.
    pub spawn: Spawn,
    /// Hands each text to the typer's thread.
    pub typist: Typist,
}
