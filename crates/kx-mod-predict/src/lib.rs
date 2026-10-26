//! Word prediction (spec §6, ADR-0032): the module that shows word chips, types picks through the keyboard and learns
//! on the device, over pure parts: folding and casing, the word splitting the pack builder shares, the word-pack
//! format, the tracker, the scorer, the learner and email suggestions.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod actor;
pub mod casing;
mod days;
pub mod email;
pub mod fold;
mod lane;
pub mod learn;
mod manifest;
mod menu;
pub mod pack;
pub mod score;
pub mod script;
mod settings;
pub mod split;
pub mod track;

pub use actor::{
    ASIDE_SUFFIX, Actor, EMAILS_BLOB, Flow, Msg, Ports, STOPPED_LINE, aside_blob, emails_blob,
    learned_blob, watch,
};
pub use casing::Casing;
pub use days::{Days, SystemDays};
pub use fold::fold;
pub use manifest::{ID, MANIFEST};
pub use pack::{Flags, Pack, PackFormatError, WordId};
#[cfg(feature = "build")]
pub use pack::{PackWriter, WriteError};
pub use settings::{LEARN_KEY, MAX_WORD, ON_KEY, PredictSettings, SPEC};

use kx_module_api::ports::{
    ChipPicked, FieldWatchKey, Lane, LayoutChanged, PointerPressed, PressesAsk, PressesWanted,
    SealedStoreKey, Suggested, Typed, TypingMode, WordPacksKey, WordsAsk,
};
use kx_module_api::{Bus, Event, EventError, Manifest, Module, ModuleCx, ModuleError, keys};
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

/// The actor thread's name, as a debugger shows it.
const THREAD: &str = "kx-predict";
/// Why the start failed when the actor's thread did not start.
const NO_THREAD: &str = "the predict actor's thread did not start";

/// The prediction module: it reads `[predict]`, watches the focused box and runs its actor.
#[derive(Default)]
pub struct PredictModule {
    /// The running actor's inbox and thread, from start to stop.
    actor: Option<(Sender<Msg>, JoinHandle<()>)>,
}

impl Module for PredictModule {
    fn manifest(&self) -> &'static Manifest {
        &MANIFEST
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        self.halt();
        let settings = cx.settings::<PredictSettings>()?;
        let field = cx.service::<FieldWatchKey>()?;
        let packs = cx.service::<WordPacksKey>()?;
        let store = cx.service::<SealedStoreKey>()?;
        let (tx, inbox) = mpsc::channel();
        relay(cx, &tx, |e: &Typed| Msg::Typed(e.piece.clone()))?;
        relay(cx, &tx, |e: &TypingMode| Msg::Mode(*e))?;
        relay(cx, &tx, |e: &LayoutChanged| Msg::Lang {
            iso: e.iso.clone(),
            name: e.name.clone(),
            rtl: e.rtl,
        })?;
        relay(cx, &tx, |e: &PointerPressed| Msg::Pressed(e.ours))?;
        relay(cx, &tx, |e: &ChipPicked| Msg::Picked(*e))?;
        relay(cx, &tx, |e: &WordsAsk| Msg::Ask(*e))?;
        relay(cx, &tx, |_: &PressesAsk| Msg::PressesAsk)?;
        let ports = Ports {
            bus: cx.bus(),
            clock: cx.clock(),
            days: Arc::new(SystemDays),
            packs,
            store,
            handle: cx.settings_handle(),
            field: watch(&*field, settings.field_config(), &tx),
        };
        let bus = cx.bus();
        let actor = Actor::new(ports, &settings);
        let thread = thread::Builder::new()
            .name(THREAD.to_owned())
            .spawn(move || run(actor, &inbox, &bus))
            .map_err(|_| ModuleError::Start(NO_THREAD))?;
        self.actor = Some((tx, thread));
        Ok(())
    }

    /// Joins the actor, so it saves what changed and lets go of the box before the module is gone.
    fn stop(&mut self) {
        self.halt();
    }
}

impl Drop for PredictModule {
    fn drop(&mut self) {
        self.halt();
    }
}

impl PredictModule {
    /// Tells the actor to quit and waits for its thread to end; a panic there is logged, with no word or email.
    fn halt(&mut self) {
        if let Some((tx, thread)) = self.actor.take() {
            let _ = tx.send(Msg::Quit);
            if thread.join().is_err() {
                tracing::warn!("predict actor panicked");
            }
        }
    }
}

/// The actor's thread. A panic there empties the lane, lets presses go and tells the user; then `halt` logs it.
fn run(actor: Actor, inbox: &Receiver<Msg>, bus: &Bus) {
    if let Err(panic) = catch_unwind(AssertUnwindSafe(|| actor.run(inbox))) {
        let empty = Suggested {
            round: 0,
            lane: Lane::Empty,
            forgetting: false,
            learn: false,
            on: false,
            list: None,
        };
        // The module holds every gate these need, so none is refused.
        let _ = bus.publish(&empty);
        let _ = bus.publish(&PressesWanted { on: false });
        let _ = bus.publish(&actor::notice(keys::MODULE_FAILED, Vec::new()));
        resume_unwind(panic);
    }
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
