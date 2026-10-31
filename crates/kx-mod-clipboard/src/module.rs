//! The module: it reads `[clipboard]`, loads the history, starts the watch, the helper threads and its actor on
//! `kx-clip`. A locked shelf or a newer version's list fails the start with nothing changed (edge 15), so an older
//! app never spoils a newer history.

use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use kx_module_api::ports::{
    CalendarKey, ClipAct, ClipError, ClipKey, ClipOpen, ClipPanel, Clipboard, ClipboardKey,
    ClipboardWriterKey, HistoryAsk, HistoryWanted, Pasted, Pictures, PicturesKey, SealedError,
    SealedShelfKey,
};
use kx_module_api::{Bus, Event, EventError, Manifest, Module, ModuleCx, ModuleError, keys};

use crate::MANIFEST;
use crate::actor::{Actor, Msg, Ports, Watched, notice};
use crate::media::{Job, Look, spawn_files, spawn_media};
use crate::settings::ClipSettings;
use crate::store::{Store, StoreError};

/// The actor thread's name, as a debugger shows it.
const THREAD: &str = "kx-clip";
/// Why the start failed, as short fixed texts with no user data.
const NO_THREAD: &str = "a clipboard thread did not start";
const NO_WATCH: &str = "the clipboard cannot be watched";
const LOCKED: &str = "the clipboard history could not be unlocked";
const NEWER: &str = "the clipboard history was saved by a newer version";
const NO_LOAD: &str = "the clipboard history could not be read";

/// The running actor's inbox and threads.
struct Running {
    tx: Sender<Msg>,
    actor: JoinHandle<()>,
    media: JoinHandle<()>,
}

/// The helper threads' inboxes, and the media thread that `halt` joins.
struct Helpers {
    media: Sender<Job>,
    files: Sender<Look>,
    drawing: JoinHandle<()>,
}

/// The clipboard history module: it watches copies, keeps them sealed, and runs the panel and insert.
#[derive(Default)]
pub struct ClipboardModule {
    run: Option<Running>,
}

impl Module for ClipboardModule {
    fn manifest(&self) -> &'static Manifest {
        &MANIFEST
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        self.halt();
        let settings = cx.settings::<ClipSettings>()?;
        let clip = cx.service::<ClipboardKey>()?;
        let writer = cx.service::<ClipboardWriterKey>()?;
        let sealed = cx.service::<SealedShelfKey>()?;
        let pictures = cx.service::<PicturesKey>()?;
        let calendar = cx.service::<CalendarKey>()?;
        let mut store = Store::new(Arc::clone(&sealed), settings.store_config());
        let loaded = store.load().map_err(refuse)?;
        let (tx, inbox) = mpsc::channel();
        let (watch, copies) = Watched::start(&*clip, &settings, &tx).map_err(no_watch)?;
        let Helpers {
            media,
            files,
            drawing,
        } = helpers(&clip, &*pictures, &settings, &tx)?;
        relays(cx, &tx)?;
        let ports = Ports {
            bus: cx.bus(),
            clock: cx.clock(),
            calendar,
            shelf: sealed,
            writer,
            media,
            files,
            copies,
            watch: Some(watch),
        };
        let (actor, bus) = (Actor::new(ports, &settings, store, loaded), cx.bus());
        let thread = thread::Builder::new().name(THREAD.to_owned());
        let actor = thread
            .spawn(move || run_actor(actor, &inbox, &bus))
            .map_err(|e| no_thread(&e))?;
        self.run = Some(Running {
            tx,
            actor,
            media: drawing,
        });
        Ok(())
    }

    /// Joins the actor, so it saves and lets go of the watch, then the media thread, which stops the worker.
    fn stop(&mut self) {
        self.halt();
    }
}

impl Drop for ClipboardModule {
    fn drop(&mut self) {
        self.halt();
    }
}

impl ClipboardModule {
    /// Tells the actor to quit and waits for its threads; a panic there is logged, with no content.
    fn halt(&mut self) {
        let Some(run) = self.run.take() else {
            return;
        };
        let _ = run.tx.send(Msg::Quit);
        if run.actor.join().is_err() {
            tracing::warn!("clipboard actor panicked");
        }
        if run.media.join().is_err() {
            tracing::warn!("clipboard media thread panicked");
        }
    }
}

/// The actor's thread. A panic there closes the panel, frees the chord and tells the user; then `halt` logs it.
fn run_actor(actor: Actor, inbox: &Receiver<Msg>, bus: &Bus) {
    if let Err(panic) = catch_unwind(AssertUnwindSafe(|| actor.run(inbox))) {
        // The module holds every gate these need, so none is refused.
        let _ = bus.publish(&ClipPanel { open: false });
        let _ = bus.publish(&HistoryWanted { on: false });
        let _ = bus.publish(&notice(keys::MODULE_FAILED));
        resume_unwind(panic);
    }
}

/// Why a load failed, as the start's error; the kernel tells the user and offers Try again, so the module adds no
/// notice of its own.
fn refuse(error: StoreError) -> ModuleError {
    tracing::warn!(%error, "the clipboard history did not load");
    match error {
        StoreError::Newer => ModuleError::Start(NEWER),
        StoreError::Shelf(SealedError::Locked) => ModuleError::Start(LOCKED),
        StoreError::Shelf(_) => ModuleError::Start(NO_LOAD),
    }
}

/// The start's error when a thread did not start.
fn no_thread(error: &std::io::Error) -> ModuleError {
    tracing::warn!(%error, "a clipboard thread did not start");
    ModuleError::Start(NO_THREAD)
}

/// The start's error when the OS refuses the watch.
fn no_watch(error: ClipError) -> ModuleError {
    tracing::warn!(%error, "the clipboard cannot be watched");
    ModuleError::Start(NO_WATCH)
}

/// Starts the helper threads, each telling the actor on `tx`: pictures on `kx-clip-media`, which `halt` joins, and
/// file checks on `kx-clip-files`. Returns their inboxes and the media thread.
fn helpers(
    clip: &Arc<dyn Clipboard>,
    pictures: &dyn Pictures,
    settings: &ClipSettings,
    tx: &Sender<Msg>,
) -> Result<Helpers, ModuleError> {
    let (media, jobs) = mpsc::channel();
    let run = pictures.open(settings.picture_config());
    let drawing = spawn_media(run, settings.sides(), jobs, sender(tx, Msg::Drawn))
        .map_err(|e| no_thread(&e))?;
    let (files, looks) = mpsc::channel();
    // Never joined: a share out of reach may hold it, and it ends once the actor lets go.
    drop(spawn_files(Arc::clone(clip), looks, sender(tx, Msg::Facts)).map_err(|e| no_thread(&e))?);
    Ok(Helpers {
        media,
        files,
        drawing,
    })
}

/// A callback handing each `T` to the actor as `msg` wraps it; a stopped actor misses it.
fn sender<T: 'static>(tx: &Sender<Msg>, msg: fn(T) -> Msg) -> impl Fn(T) + Send + 'static {
    let tx = tx.clone();
    move |t| drop(tx.send(msg(t)))
}

/// Hands the panel's and the keyboard's events to the actor.
fn relays(cx: &mut ModuleCx<'_>, tx: &Sender<Msg>) -> Result<(), EventError> {
    relay(cx, tx, |_: &ClipOpen| Msg::Open)?;
    relay(cx, tx, |e: &ClipKey| Msg::Key(e.0.clone()))?;
    relay(cx, tx, |e: &ClipAct| Msg::Act(*e))?;
    relay(cx, tx, |e: &Pasted| Msg::Pasted(*e))?;
    relay(cx, tx, |_: &HistoryAsk| Msg::HistoryAsk)
}

/// Hands each `E` on the bus to the actor as `msg` says.
fn relay<E: Event>(
    cx: &mut ModuleCx<'_>,
    tx: &Sender<Msg>,
    msg: impl Fn(&E) -> Msg + Send + Sync + 'static,
) -> Result<(), EventError> {
    let tx = tx.clone();
    cx.subscribe(move |e: &E| {
        // Only a stopped actor misses this, and its handlers leave the bus before it stops.
        let _ = tx.send(msg(e));
    })
}
