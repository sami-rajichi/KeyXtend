//! The power module: the Power key opens the system's own power dialog (spec §8.5, ADR-0031). It reaches the OS
//! only through the platform's `PowerDialog` port, and never shuts down or confirms anything itself.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod actor;
mod manifest;

pub use actor::{Actor, Job, Msg, Spawn};
pub use manifest::{ID, MANIFEST};

use kx_module_api::ports::{PowerAsk, PowerAsked, PowerChanged, PowerDialogKey};
use kx_module_api::{Bus, Event, EventError, Manifest, Module, ModuleCx, ModuleError, keys};
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

/// The actor thread's name, as a debugger shows it.
const THREAD: &str = "kx-power";
/// The name of each short thread that opens the dialog.
const OPENER: &str = "kx-power-open";
/// Why the start failed when the actor's thread did not start.
const NO_THREAD: &str = "the power actor's thread did not start";

/// The power module: it runs its actor, which opens the dialog at each click of the Power key.
#[derive(Default)]
pub struct PowerModule {
    /// The running actor's inbox and thread, from start to stop.
    actor: Option<(Sender<Msg>, JoinHandle<()>)>,
}

impl Module for PowerModule {
    fn manifest(&self) -> &'static Manifest {
        &MANIFEST
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        self.halt();
        let dialog = cx.service::<PowerDialogKey>()?;
        let (tx, inbox) = mpsc::channel();
        relay(cx, &tx, |_: &PowerAsked| Msg::Asked)?;
        relay(cx, &tx, |_: &PowerAsk| Msg::Ask)?;
        let bus = cx.bus();
        let actor = Actor::new(dialog, bus.clone(), tx.clone(), detached);
        let thread = thread::Builder::new()
            .name(THREAD.to_owned())
            .spawn(move || run(actor, &inbox, &bus))
            .map_err(|_| ModuleError::Start(NO_THREAD))?;
        self.actor = Some((tx, thread));
        Ok(())
    }

    /// Joins the actor, which no open holds, so the stop never waits on the dialog.
    fn stop(&mut self) {
        self.halt();
    }
}

impl Drop for PowerModule {
    fn drop(&mut self) {
        self.halt();
    }
}

impl PowerModule {
    /// Tells the actor to quit and waits for its thread to end; a panic there is logged.
    fn halt(&mut self) {
        if let Some((tx, thread)) = self.actor.take() {
            let _ = tx.send(Msg::Quit);
            if thread.join().is_err() {
                tracing::warn!("power actor panicked");
            }
        }
    }
}

/// The actor's thread. A panic there says at once that the module is off, so the Power key dims, and tells the user
/// to restart; then `halt` logs it.
fn run(actor: Actor, inbox: &Receiver<Msg>, bus: &Bus) {
    if let Err(panic) = catch_unwind(AssertUnwindSafe(|| actor.run(inbox))) {
        // Neither event needs a capability, so neither can be refused.
        let _ = bus.publish(&PowerChanged::OFF);
        let _ = bus.publish(&actor::notice(keys::POWER_STOPPED));
        resume_unwind(panic);
    }
}

/// Runs `job` on a short thread that nothing joins, so a dialog that holds the call holds only that thread.
fn detached(job: Job) -> bool {
    thread::Builder::new()
        .name(OPENER.to_owned())
        .spawn(job)
        .is_ok()
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
