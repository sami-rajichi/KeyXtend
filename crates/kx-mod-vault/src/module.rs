//! The module: it reads `[vault]`, loads the notebook and the backup's state from its own sealed store, starts the
//! box job thread and runs its actor on `kx-vault`. A damaged notebook or backup state starts empty with its notice; a
//! locked or failing store, or a newer version's blob, fails the start with nothing written, and the kernel tells it
//! and offers Try again.

use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use kx_module_api::ports::{
    BoxesKey, CalendarKey, ClipTaken, FillAct, FillAsk, FillClose, FillKey, FillLive, FillNames,
    FillOpen, FillPanel, FillTyped, PlacesKey, RandomKey, SealedError, VaultEmails, VaultStoreKey,
};
use kx_module_api::{
    Bus, Event, EventError, Manifest, Module, ModuleCx, ModuleError, Notice, NoticeAnswered, keys,
};

use crate::actor::{Actor, Msg, Ports, Stores};
use crate::backup::{BackupStore, Jobs, Links, Runner};
use crate::settings::VaultSettings;
use crate::store::{Store, StoreError};
use crate::{ID, MANIFEST};

/// The actor thread's name, as a debugger shows it.
const THREAD: &str = "kx-vault";
/// Why the start failed, as short fixed texts with no user data.
const NO_THREAD: &str = "the vault thread did not start";
const LOCKED: &str = "the notebook could not be unlocked";
const NEWER: &str = "the notebook was saved by a newer version";
const BACKUP_NEWER: &str = "the backup state was saved by a newer version";
const NO_LOAD: &str = "the notebook could not be read";

/// The running actor's inbox and thread.
struct Running {
    tx: Sender<Msg>,
    actor: JoinHandle<()>,
}

/// The Quick-fill module: it keeps the notebook and runs the panel.
#[derive(Default)]
pub struct VaultModule {
    run: Option<Running>,
}

impl Module for VaultModule {
    fn manifest(&self) -> &'static Manifest {
        &MANIFEST
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        self.halt();
        let settings = cx.settings::<VaultSettings>()?;
        let ports = Ports {
            bus: cx.bus(),
            clock: cx.clock(),
            calendar: cx.service::<CalendarKey>()?,
            random: cx.service::<RandomKey>()?,
        };
        let stores = load(cx)?;
        let (boxes, places) = (cx.service::<BoxesKey>()?, cx.service::<PlacesKey>()?);
        let (tx, inbox) = mpsc::channel();
        relays(cx, &tx)?;
        let runner = Runner::new(boxes.start(settings.box_config()), places);
        let reply = tx.clone();
        let jobs = Jobs::spawn(runner, move |done| drop(reply.send(Msg::Done(done))));
        let jobs = Box::new(jobs.map_err(|e| no_thread(&e))?);
        let handle = cx.settings_handle();
        let links = Links { jobs, handle };
        let (actor, bus) = (Actor::new(ports, links, &settings, stores), cx.bus());
        let thread = thread::Builder::new().name(THREAD.to_owned());
        let actor = thread
            .spawn(move || run_actor(actor, &inbox, &bus))
            .map_err(|e| no_thread(&e))?;
        self.run = Some(Running { tx, actor });
        Ok(())
    }

    /// Joins the actor, so it saves what waits and closes the panel.
    fn stop(&mut self) {
        self.halt();
    }
}

impl Drop for VaultModule {
    fn drop(&mut self) {
        self.halt();
    }
}

impl VaultModule {
    /// True while the actor runs.
    #[must_use]
    pub fn running(&self) -> bool {
        self.run.is_some()
    }

    /// Tells the actor to quit and waits for it; a panic there is logged, with no value.
    fn halt(&mut self) {
        let Some(run) = self.run.take() else {
            return;
        };
        let _ = run.tx.send(Msg::Quit);
        if run.actor.join().is_err() {
            tracing::warn!("vault actor panicked");
        }
    }
}

/// Loads the notebook and the backup's state; each damaged one shows its notice.
fn load(cx: &ModuleCx<'_>) -> Result<Stores, ModuleError> {
    let sealed = cx.service::<VaultStoreKey>()?;
    let (mut store, mut backup) = (Store::new(Arc::clone(&sealed)), BackupStore::new(sealed));
    let loaded = store.load().map_err(|e| refuse(e, NEWER))?;
    let kept = backup.load().map_err(|e| refuse(e, BACKUP_NEWER))?;
    for (damaged, key) in [
        (loaded.damaged, keys::VAULT_DAMAGED),
        (kept.damaged, keys::VAULT_BACKUP_DAMAGED),
    ] {
        if damaged {
            // A notice needs no gate, so none is refused.
            let _ = cx.bus().publish(&Notice::about(key, ID));
        }
    }
    Ok(Stores {
        store,
        book: loaded.book,
        backup,
        kept: kept.kept,
        damaged: loaded.damaged,
    })
}

/// The actor's thread. A panic there closes the panel, empties the emails, says it stopped and tells the user; then
/// `halt` logs it.
fn run_actor(actor: Actor, inbox: &Receiver<Msg>, bus: &Bus) {
    if let Err(panic) = catch_unwind(AssertUnwindSafe(|| actor.run(inbox))) {
        // The module holds every gate these need, so none is refused.
        let _ = bus.publish(&FillPanel { open: false });
        let _ = bus.publish(&VaultEmails { emails: Vec::new() });
        let _ = bus.publish(&FillLive::OFF);
        let _ = bus.publish(&Notice::about(keys::MODULE_FAILED, ID));
        resume_unwind(panic);
    }
}

/// Why a load failed, as the start's error, `newer` for a newer version's blob; only the error's kind is logged.
fn refuse(error: StoreError, newer: &'static str) -> ModuleError {
    tracing::warn!(%error, "the vault's state did not load");
    match error {
        StoreError::Newer => ModuleError::Start(newer),
        StoreError::Sealed(SealedError::Locked) => ModuleError::Start(LOCKED),
        StoreError::Sealed(_) | StoreError::Held => ModuleError::Start(NO_LOAD),
    }
}

/// The start's error when a thread did not start.
fn no_thread(error: &std::io::Error) -> ModuleError {
    tracing::warn!(%error, "a vault thread did not start");
    ModuleError::Start(NO_THREAD)
}

/// Hands the keyboard's, the ui's, prediction's and the clipboard history's events to the actor, and the answers to
/// the module's own notices.
fn relays(cx: &mut ModuleCx<'_>, tx: &Sender<Msg>) -> Result<(), EventError> {
    relay(cx, tx, |_: &FillOpen| Msg::Open)?;
    relay(cx, tx, |_: &FillClose| Msg::Close)?;
    relay(cx, tx, |_: &FillAsk| Msg::Ask)?;
    relay(cx, tx, |e: &FillKey| Msg::Key(e.0.clone()))?;
    relay(cx, tx, |e: &FillAct| Msg::Act(*e))?;
    relay(cx, tx, |e: &FillTyped| Msg::Typed(*e))?;
    relay(cx, tx, |e: &ClipTaken| Msg::Taken(e.clone()))?;
    relay(cx, tx, |e: &FillNames| Msg::Names(e.0.clone()))?;
    let tx = tx.clone();
    cx.subscribe(move |e: &NoticeAnswered| {
        if e.notice.module == Some(ID) {
            let (key, yes) = (e.notice.key, e.yes);
            // Only a stopped actor misses this, and its handlers leave the bus before it stops.
            let _ = tx.send(Msg::Answered { key, yes });
        }
    })
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
