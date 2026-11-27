//! The module: it reads `[snip]`, takes its services through the manifest's gates, and starts its actor on `kx-snip`,
//! handing it the Snip key, the windows' clicks and the keyboard's asks and keys. The videos a delete missed are kept for the
//! module's next start only; after an app restart the sweep deletes every video not marked copied.

use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use kx_module_api::ports::{
    CalendarKey, ClipboardWriterKey, FileAskKey, FolderRevealKey, MediaOpenKey, MicPassKey,
    PicturesKey, RecorderKey, ScreenFreezeKey, SnipAct, SnipAside, SnipAsk, SnipFolderKey,
    SnipLine, SnipLineKey, SnipLit, SnipPage, SnipStage, SnipStart, SnipView,
};
use kx_module_api::{
    Bus, Event, EventError, Manifest, Module, ModuleCx, ModuleError, Notice, keys,
};

use crate::actor::{Actor, Job, Leftovers, Msg, Ports};
use crate::settings::SnipSettings;
use crate::{ID, MANIFEST};

/// The actor thread's name, as a debugger shows it.
const THREAD: &str = "kx-snip";
/// The name of each short thread that runs a job.
const WORKER: &str = "kx-snip-job";
/// Why the start failed when the actor's thread did not start.
const NO_THREAD: &str = "the snip actor's thread did not start";

/// The snip module.
#[derive(Debug, Default)]
pub struct SnipModule {
    /// The running actor's inbox and thread, from start to stop.
    run: Option<(Sender<Msg>, JoinHandle<()>)>,
    /// The videos a delete missed, which the module's next start sweeps; they live as long as the app.
    leftovers: Leftovers,
}

impl SnipModule {
    /// A module that has not started.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Tells the actor to quit and waits for its thread; a panic there is logged.
    fn halt(&mut self) {
        if let Some((tx, thread)) = self.run.take() {
            let _ = tx.send(Msg::Quit);
            if thread.join().is_err() {
                tracing::warn!("snip actor panicked");
            }
        }
    }
}

impl Module for SnipModule {
    fn manifest(&self) -> &'static Manifest {
        &MANIFEST
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        self.halt();
        let settings = cx.settings::<SnipSettings>()?;
        let (tx, inbox) = mpsc::channel();
        let ports = ports(cx, &tx, &self.leftovers)?;
        relays(cx, &tx)?;
        let bus = cx.bus();
        let actor = Actor::new(ports, settings);
        let thread = thread::Builder::new()
            .name(THREAD.to_owned())
            .spawn(move || run_actor(actor, &inbox, &bus))
            .map_err(|_| ModuleError::Start(NO_THREAD))?;
        self.run = Some((tx, thread));
        Ok(())
    }

    /// Joins the actor, which ends the snip as Quit does; a job still out ends on its own.
    fn stop(&mut self) {
        self.halt();
    }
}

impl Drop for SnipModule {
    fn drop(&mut self) {
        self.halt();
    }
}

/// The actor's ports: every service through the manifest's gates.
fn ports(
    cx: &mut ModuleCx<'_>,
    tx: &Sender<Msg>,
    leftovers: &Leftovers,
) -> Result<Ports, ModuleError> {
    Ok(Ports {
        bus: cx.bus(),
        clock: cx.clock(),
        calendar: cx.service::<CalendarKey>()?,
        freeze: cx.service::<ScreenFreezeKey>()?,
        recorder: cx.service::<RecorderKey>()?,
        mics: cx.service::<MicPassKey>()?,
        folder: cx.service::<SnipFolderKey>()?,
        dialogs: cx.service::<FileAskKey>()?,
        player: cx.service::<MediaOpenKey>()?,
        clip: cx.service::<ClipboardWriterKey>()?,
        pictures: cx.service::<PicturesKey>()?,
        reveal: cx.service::<FolderRevealKey>()?,
        handle: cx.settings_handle(),
        back: tx.clone(),
        spawn: detached,
        leftovers: leftovers.clone(),
    })
}

/// The actor's thread. A panic there drops the actor, whose runs are cancelled and their videos deleted, then closes the
/// windows, darkens the key, brings the keyboard back and tells the user; `halt` logs it.
fn run_actor(actor: Actor, inbox: &Receiver<Msg>, bus: &Bus) {
    if let Err(panic) = catch_unwind(AssertUnwindSafe(|| actor.run(inbox))) {
        let page = SnipPage {
            round: 0,
            frozen: Vec::new(),
            stage: SnipStage::Idle,
            ask: None,
            note: None,
            panel: None,
            dialog: false,
        };
        // The module holds `Snip`, which these events need, or its start would have failed.
        let _ = bus.publish(&SnipView::new(page));
        let _ = bus.publish(&SnipLit::Off);
        let _ = bus.publish(&SnipAside::Back);
        let _ = bus.publish(&SnipLine(None));
        let _ = bus.publish(&Notice::about(keys::MODULE_FAILED, ID));
        resume_unwind(panic);
    }
}

/// Runs `job` on a short thread that nothing joins, so a slow worker or dialog holds only that thread.
fn detached(job: Job) -> bool {
    thread::Builder::new()
        .name(WORKER.to_owned())
        .spawn(job)
        .is_ok()
}

/// Hands the Snip key, the windows' clicks, the keyboard's ask and its keys to the actor; a key keeps its letters
/// redacted.
fn relays(cx: &mut ModuleCx<'_>, tx: &Sender<Msg>) -> Result<(), EventError> {
    relay(cx, tx, |_: &SnipStart| Msg::Key)?;
    relay(cx, tx, |e: &SnipAct| Msg::Act(e.clone()))?;
    relay(cx, tx, |_: &SnipAsk| Msg::Ask)?;
    relay(cx, tx, |e: &SnipLineKey| Msg::LineKey(e.0.clone()))
}

/// Hands each `E` on the bus to the actor as `msg` says.
fn relay<E: Event>(
    cx: &mut ModuleCx<'_>,
    tx: &Sender<Msg>,
    msg: impl Fn(&E) -> Msg + Send + Sync + 'static,
) -> Result<(), EventError> {
    let tx = tx.clone();
    // Only a stopped actor misses this, and its handlers leave the bus before it stops.
    cx.subscribe(move |e: &E| drop(tx.send(msg(e))))
}
