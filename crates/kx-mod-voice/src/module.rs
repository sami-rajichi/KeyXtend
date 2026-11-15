//! The module: it reads `[voice]`, drops a bad pick alone with the settings notice, and with `on` starts the watches,
//! the typer and its actor on `kx-voice`, handing the bus's events to it. With `on` off it shows no bar, lights no key
//! and asks for no service, so the worker never starts (edge 36).

use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use kx_module_api::ports::{
    CalendarKey, ClipboardWriterKey, FieldRun, FieldWatchKey, FolderRevealKey, LayoutChanged,
    ModelFilesKey, PointerPressed, PressAsker, PressesAsk, PressesWanted, SessionRun,
    SessionWatchKey, SpeechKey, TextDone, Typed, VoiceAct, VoiceAsk, VoiceBar, VoiceKey, VoiceLit,
    VoiceOpen, VoicePage, VoiceShelfKey, VoiceView,
};
use kx_module_api::{
    Bus, Event, EventError, Manifest, Module, ModuleCx, ModuleError, Notice, NoticeArg, args, keys,
};

use crate::actor::{Actor, Job, Msg, Ports, typer};
use crate::settings::{PICKS, VoiceSettings};
use crate::{ID, MANIFEST};

/// The actor thread's name, as a debugger shows it.
const THREAD: &str = "kx-voice";
/// The name of each short thread that runs a job.
const WORKER: &str = "kx-voice-job";
/// Why the start failed when the actor's thread did not start.
const NO_THREAD: &str = "the voice actor's thread did not start";
/// Why the start failed when the typer's thread did not start.
const NO_TYPER: &str = "the voice typer's thread did not start";
/// The round of the view an idle or failed module tells, before any of the flow's.
const HIDDEN_ROUND: u32 = 0;

/// The voice typing module.
#[derive(Debug, Default)]
pub struct VoiceModule {
    /// The run's saves would be hidden (ADR-0037), so nothing is kept.
    hidden: bool,
    /// The running actor's inbox and thread, from start to stop.
    run: Option<(Sender<Msg>, JoinHandle<()>)>,
}

impl VoiceModule {
    /// A module that has not started.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A module for a run whose saves would be hidden (ADR-0037): it keeps no text, recording or count.
    #[must_use]
    pub fn hidden() -> Self {
        Self {
            hidden: true,
            run: None,
        }
    }

    /// Tells the actor to quit and waits for its thread, which no job or text holds; a panic there is logged.
    fn halt(&mut self) {
        if let Some((tx, thread)) = self.run.take() {
            let _ = tx.send(Msg::Quit);
            if thread.join().is_err() {
                tracing::warn!("voice actor panicked");
            }
        }
    }
}

impl Module for VoiceModule {
    fn manifest(&self) -> &'static Manifest {
        &MANIFEST
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        self.halt();
        let settings = read(cx)?;
        if !settings.on {
            hide(&cx.bus(), &settings);
            return Ok(());
        }
        let (tx, inbox) = mpsc::channel();
        let ports = ports(cx, &settings, &tx)?;
        relays(cx, &tx)?;
        let bus = cx.bus();
        let actor = Actor::new(ports, settings.clone(), self.hidden);
        let thread = thread::Builder::new()
            .name(THREAD.to_owned())
            .spawn(move || run_actor(actor, &inbox, &bus, &settings))
            .map_err(|_| ModuleError::Start(NO_THREAD))?;
        self.run = Some((tx, thread));
        Ok(())
    }

    /// Joins the actor, which drops the watches, closes every run and stops the typer; a run out in a job closes as
    /// that job ends.
    fn stop(&mut self) {
        self.halt();
    }
}

impl Drop for VoiceModule {
    fn drop(&mut self) {
        self.halt();
    }
}

/// `[voice]` with each bad pick dropped alone (edge case 19): the settings notice names it, and the rest is written
/// back so the next start finds it gone.
fn read(cx: &ModuleCx<'_>) -> Result<VoiceSettings, ModuleError> {
    let mut settings = cx.settings::<VoiceSettings>()?;
    let dropped = settings.drop_bad_picks();
    if dropped.is_empty() {
        return Ok(settings);
    }
    if let Ok(picks) = toml::Value::try_from(&settings.picks) {
        // A failed save keeps the repair for this run, and the kernel tells the user about the file.
        let _ = cx.settings_handle().set(PICKS, picks);
    }
    let mut notice = Notice::about(keys::SETTINGS_RESET, ID);
    notice.args.push((args::KEYS, NoticeArg::List(dropped)));
    // A notice needs no gate, so none is refused.
    let _ = cx.bus().publish(&notice);
    Ok(settings)
}

/// The actor's ports: every service through the manifest's gates, the watches telling the inbox, and the typer.
fn ports(
    cx: &mut ModuleCx<'_>,
    settings: &VoiceSettings,
    tx: &Sender<Msg>,
) -> Result<Ports, ModuleError> {
    let speech = cx.service::<SpeechKey>()?;
    let files = cx.service::<ModelFilesKey>()?;
    let clip = cx.service::<ClipboardWriterKey>()?;
    let reveal = cx.service::<FolderRevealKey>()?;
    let shelf = cx.service::<VoiceShelfKey>()?;
    let calendar = cx.service::<CalendarKey>()?;
    let field = field_watch(cx, settings, tx)?;
    let session = session_watch(cx, tx)?;
    let typist = typer(cx.bus()).ok_or(ModuleError::Start(NO_TYPER))?;
    Ok(Ports {
        bus: cx.bus(),
        clock: cx.clock(),
        calendar,
        speech,
        files,
        clip,
        reveal,
        shelf,
        field,
        session,
        handle: cx.settings_handle(),
        back: tx.clone(),
        spawn: detached,
        typist,
    })
}

/// The focused box's watch, whose tell only posts its kind and box; `None`, logged by kind, when it is refused.
fn field_watch(
    cx: &ModuleCx<'_>,
    settings: &VoiceSettings,
    tx: &Sender<Msg>,
) -> Result<Option<Box<dyn FieldRun>>, ModuleError> {
    let watch = cx.service::<FieldWatchKey>()?;
    let tx = tx.clone();
    // Only a stopped actor misses a change, and its watch goes with it.
    let tell = Box::new(move |change| drop(tx.send(Msg::Field(change))));
    let run = watch.watch(settings.field_config(), tell);
    let run = run.inspect_err(|error| tracing::warn!(%error, "the focused box cannot be watched"));
    Ok(run.ok())
}

/// The session's watch, whose tell only posts its note; `None`, logged by kind, when it is refused.
fn session_watch(
    cx: &ModuleCx<'_>,
    tx: &Sender<Msg>,
) -> Result<Option<Box<dyn SessionRun>>, ModuleError> {
    let watch = cx.service::<SessionWatchKey>()?;
    let tx = tx.clone();
    let tell = Box::new(move |note| drop(tx.send(Msg::Session(note))));
    let run = watch.watch(tell);
    let run = run.inspect_err(|error| tracing::warn!(%error, "the session cannot be watched"));
    Ok(run.ok())
}

/// Shows no bar and a dark key, as an idle or failed module does.
fn hide(bus: &Bus, settings: &VoiceSettings) {
    let page = VoicePage {
        round: HIDDEN_ROUND,
        bar: VoiceBar::Hidden,
        panel: None,
        place: settings.bar_place(),
        sounds: settings.sounds,
    };
    // The module holds `Voice`, which these events need, or its start would have failed.
    let _ = bus.publish(&VoiceView::new(page));
    let _ = bus.publish(&VoiceLit::OFF);
}

/// The actor's thread. A panic there hides the bar, darkens the key, lets presses go and tells the user; the runs it
/// held closed as it unwound, and then `halt` logs it.
fn run_actor(actor: Actor, inbox: &Receiver<Msg>, bus: &Bus, settings: &VoiceSettings) {
    if let Err(panic) = catch_unwind(AssertUnwindSafe(|| actor.run(inbox))) {
        hide(bus, settings);
        let _ = bus.publish(&PressesWanted {
            asker: PressAsker::Voice,
            on: false,
        });
        let _ = bus.publish(&Notice::about(keys::MODULE_FAILED, ID));
        resume_unwind(panic);
    }
}

/// Runs `job` on a short thread that nothing joins, so a slow worker holds only that thread.
fn detached(job: Job) -> bool {
    thread::Builder::new()
        .name(WORKER.to_owned())
        .spawn(job)
        .is_ok()
}

/// Hands the keyboard's, the ui's, the layouts' and the mouse module's events to the actor; a typed key or a click
/// outside our windows reaches it only as that, never with its text.
fn relays(cx: &mut ModuleCx<'_>, tx: &Sender<Msg>) -> Result<(), EventError> {
    relay(cx, tx, |_: &VoiceKey| Some(Msg::Key))?;
    relay(cx, tx, |e: &VoiceAct| Some(Msg::Act(e.clone())))?;
    relay(cx, tx, |_: &VoiceOpen| Some(Msg::Open))?;
    relay(cx, tx, |_: &VoiceAsk| Some(Msg::Ask))?;
    relay(cx, tx, |e: &LayoutChanged| Some(Msg::Layout(e.clone())))?;
    relay(cx, tx, |e: &TextDone| Some(Msg::Typed(*e)))?;
    relay(cx, tx, |_: &Typed| Some(Msg::Elsewhere))?;
    relay(cx, tx, |e: &PointerPressed| {
        (!e.ours).then_some(Msg::Elsewhere)
    })?;
    relay(cx, tx, |_: &PressesAsk| Some(Msg::PressesAsk))
}

/// Hands each `E` on the bus to the actor as `msg` says, when it says one.
fn relay<E: Event>(
    cx: &mut ModuleCx<'_>,
    tx: &Sender<Msg>,
    msg: impl Fn(&E) -> Option<Msg> + Send + Sync + 'static,
) -> Result<(), EventError> {
    let tx = tx.clone();
    cx.subscribe(move |e: &E| {
        // Only a stopped actor misses this, and its handlers leave the bus before it stops.
        if let Some(msg) = msg(e) {
            let _ = tx.send(msg);
        }
    })
}
