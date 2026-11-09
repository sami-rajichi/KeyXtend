//! The module: it reads `[selection]` and, with `on`, starts the watch and hands the bus's events to its actor on
//! `kx-selection`; with `on` off or a refused watch it stays idle.

use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use kx_module_api::ports::{
    AimedLine, AssistChanged, BoardPlace, ChipAct, ChipsView, EditDone, GrabState, PanelLine,
    PointerPressed, PointerReleased, PressesAsk, PressesWanted, SelRun, SelectionWatch,
    SelectionWatchKey, Typed, WebLinkKey,
};
use kx_module_api::{
    Bus, Event, EventError, Manifest, Module, ModuleCx, ModuleError, Notice, NoticeAnswered, keys,
};

use crate::actor::{Actor, Job, Msg, Ports};
use crate::manifest::ASKER;
use crate::settings::SelSettings;
use crate::{ID, MANIFEST};

/// The actor thread's name, as a debugger shows it.
const THREAD: &str = "kx-selection";
/// The name of each short thread that opens a link.
const OPENER: &str = "kx-selection-open";
/// Why the start failed when the actor's thread did not start.
const NO_THREAD: &str = "the selection actor's thread did not start";

/// The selection chips module: Copy, Cut, Paste and Translate keycaps by the focused box's selection.
pub struct SelectionModule {
    /// The system's display language, which Translate targets when `translate_to` is empty.
    os: String,
    /// The running actor's inbox and thread, from start to stop.
    run: Option<(Sender<Msg>, JoinHandle<()>)>,
}

impl SelectionModule {
    /// A module whose Translate follows the system's display language `os`, a BCP 47 tag such as `fr-FR`.
    #[must_use]
    pub fn new(os: &str) -> Self {
        Self {
            os: os.to_owned(),
            run: None,
        }
    }

    /// Tells the actor to quit and waits for its thread, which no open holds; a panic there is logged, with no text.
    fn halt(&mut self) {
        if let Some((tx, thread)) = self.run.take() {
            let _ = tx.send(Msg::Quit);
            if thread.join().is_err() {
                tracing::warn!("selection actor panicked");
            }
        }
    }
}

impl Module for SelectionModule {
    fn manifest(&self) -> &'static Manifest {
        &MANIFEST
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        self.halt();
        let settings = cx.settings::<SelSettings>()?;
        if !settings.on {
            idle(cx);
            return Ok(());
        }
        let watch = cx.service::<SelectionWatchKey>()?;
        let web = cx.service::<WebLinkKey>()?;
        // The relays' own gate, so a missing one watches nothing; `ReadFocusedText` was checked with the watch.
        cx.bus().check::<ChipAct>()?;
        let (tx, inbox) = mpsc::channel();
        // The watch starts first, so a refused one leaves no relay; the actor asks every state again anyway.
        let Some(run) = watched(&*watch, &settings, &tx) else {
            idle(cx);
            return Ok(());
        };
        relays(cx, &tx)?;
        let ports = Ports {
            bus: cx.bus(),
            clock: cx.clock(),
            run: Some(run),
            web,
            handle: cx.settings_handle(),
            back: tx.clone(),
            spawn: detached,
        };
        let (actor, bus) = (Actor::new(ports, &settings, &self.os), cx.bus());
        let thread = thread::Builder::new()
            .name(THREAD.to_owned())
            .spawn(move || run_actor(actor, &inbox, &bus))
            .map_err(|_| ModuleError::Start(NO_THREAD))?;
        self.run = Some((tx, thread));
        Ok(())
    }

    /// Joins the actor, so the watch stops and presses are let go before the module is gone.
    fn stop(&mut self) {
        self.halt();
    }
}

impl Drop for SelectionModule {
    fn drop(&mut self) {
        self.halt();
    }
}

/// No strip shows while the module is idle.
fn idle(cx: &ModuleCx<'_>) {
    // Without `Selection` there is no strip to hide, so a refusal changes nothing.
    let _ = cx.bus().publish(&ChipsView::Hidden);
}

/// Starts the watch, whose tell only wakes the actor; `None`, logged once with no text, when the platform refuses.
fn watched(
    watch: &dyn SelectionWatch,
    settings: &SelSettings,
    tx: &Sender<Msg>,
) -> Option<Box<dyn SelRun>> {
    let tx = tx.clone();
    // Only a stopped actor misses a wake, and its watch goes with it.
    let tell = Box::new(move || drop(tx.send(Msg::Wake)));
    let run = watch.watch(settings.sel_config(), tell);
    run.inspect_err(|error| tracing::warn!(%error, "the selection cannot be watched"))
        .ok()
}

/// The actor's thread. A panic there hides the strip, lets presses go and tells the user; then `halt` logs it.
fn run_actor(actor: Actor, inbox: &Receiver<Msg>, bus: &Bus) {
    if let Err(panic) = catch_unwind(AssertUnwindSafe(|| actor.run(inbox))) {
        // The module holds every gate these need, so none is refused.
        let _ = bus.publish(&ChipsView::Hidden);
        let _ = bus.publish(&PressesWanted {
            asker: ASKER,
            on: false,
        });
        let _ = bus.publish(&Notice::about(keys::MODULE_FAILED, ID));
        resume_unwind(panic);
    }
}

/// Runs `job` on a short thread that nothing joins, so a shell that holds the call holds only that thread.
fn detached(job: Job) -> bool {
    thread::Builder::new()
        .name(OPENER.to_owned())
        .spawn(job)
        .is_ok()
}

/// Hands the ui's, the keyboard's and the mouse module's events to the actor; the first needs `Selection`.
fn relays(cx: &mut ModuleCx<'_>, tx: &Sender<Msg>) -> Result<(), EventError> {
    relay(cx, tx, |e: &ChipAct| Msg::Act(*e))?;
    relay(cx, tx, |e: &EditDone| Msg::Done(*e))?;
    relay(cx, tx, |e: &Typed| Msg::typed(&e.piece))?;
    relay(cx, tx, |e: &AimedLine| {
        Msg::Aimed(e.line != PanelLine::None)
    })?;
    relay(cx, tx, |e: &PointerPressed| Msg::Pressed(e.ours))?;
    relay(cx, tx, |e: &PointerReleased| Msg::Released(e.ours))?;
    relay(cx, tx, |e: &AssistChanged| {
        Msg::Carrying(e.grab == GrabState::Carrying)
    })?;
    relay(cx, tx, |e: &BoardPlace| Msg::Board(e.place))?;
    relay(cx, tx, |_: &PressesAsk| Msg::PressesAsk)?;
    answers(cx, tx)
}

/// Hands the answers to the module's own notices to the actor, which acts only on its own question's.
fn answers(cx: &mut ModuleCx<'_>, tx: &Sender<Msg>) -> Result<(), EventError> {
    let tx = tx.clone();
    cx.subscribe(move |e: &NoticeAnswered| {
        if e.notice.module == Some(ID) {
            let (notice, yes) = (e.notice.clone(), e.yes);
            // Only a stopped actor misses this, and its handlers leave the bus before it stops.
            let _ = tx.send(Msg::Answered { notice, yes });
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
