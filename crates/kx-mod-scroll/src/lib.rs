//! The scroll module: the pad scrolls what the user last pointed at, gliding at two speeds until a stop (spec §5.2).
//! It moves targets only through the platform's `ScrollTarget` port (ADR-0029), so the module never talks to the OS.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod actor;
#[cfg(test)]
mod fixture;
mod glide;
mod manifest;
mod pad;
mod settings;
mod stops;

pub use actor::{Actor, Msg, STOPPED_LINE};
pub use glide::{Glide, GlideCfg};
pub use manifest::{ID, MANIFEST};
pub use pad::{Change, Pad};
pub use settings::{SPEC, ScrollSettings};
pub use stops::{Halt, Stops};

use kx_module_api::ports::{
    PointerPressed, ScrollAimed, ScrollAsk, ScrollChanged, ScrollPeek, ScrollStop, ScrollTap,
    ScrollTargetKey,
};
use kx_module_api::{Bus, Event, EventError, Manifest, Module, ModuleCx, ModuleError};
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

/// The actor thread's name, as a debugger shows it.
const THREAD: &str = "kx-scroll";
/// Why the start failed when the actor's thread did not start.
const NO_THREAD: &str = "the scroll actor's thread did not start";

/// The scroll module: it reads `[scroll]` and runs its actor, which ticks only while scrolling.
#[derive(Default)]
pub struct ScrollModule {
    /// The running actor's inbox and thread, from start to stop.
    actor: Option<(Sender<Msg>, JoinHandle<()>)>,
}

impl Module for ScrollModule {
    fn manifest(&self) -> &'static Manifest {
        &MANIFEST
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        self.halt();
        let settings = cx.settings::<ScrollSettings>()?;
        let target = cx.service::<ScrollTargetKey>()?;
        let (tx, inbox) = mpsc::channel();
        relay(cx, &tx, |e: &ScrollTap| Msg::Tap(e.dir))?;
        relay(cx, &tx, |_: &ScrollStop| Msg::Stop)?;
        relay(cx, &tx, |e: &ScrollPeek| Msg::Peek(e.on))?;
        relay(cx, &tx, |e: &PointerPressed| Msg::Pressed(e.ours))?;
        relay(cx, &tx, |_: &ScrollAsk| Msg::Ask)?;
        let bus = cx.bus();
        let actor = Actor::new(target, bus.clone(), cx.clock(), &settings);
        let thread = thread::Builder::new()
            .name(THREAD.to_owned())
            .spawn(move || run(actor, &inbox, &bus))
            .map_err(|_| ModuleError::Start(NO_THREAD))?;
        self.actor = Some((tx, thread));
        Ok(())
    }

    /// Joins the actor, so its run closes and the outline goes before the module is gone.
    fn stop(&mut self) {
        self.halt();
    }
}

impl Drop for ScrollModule {
    fn drop(&mut self) {
        self.halt();
    }
}

impl ScrollModule {
    /// Tells the actor to quit and waits for its thread to end; a panic there is logged, with no point or box.
    fn halt(&mut self) {
        if let Some((tx, thread)) = self.actor.take() {
            let _ = tx.send(Msg::Quit);
            if thread.join().is_err() {
                tracing::warn!("scroll actor panicked");
            }
        }
    }
}

/// The actor's thread. A panic there says at once that the module is not live and hides the outline, so the pad dims
/// with its tip; then `halt` logs it.
fn run(actor: Actor, inbox: &Receiver<Msg>, bus: &Bus) {
    if let Err(panic) = catch_unwind(AssertUnwindSafe(|| actor.run(inbox))) {
        // Neither event needs a capability, so neither can be refused.
        let _ = bus.publish(&ScrollChanged::OFF);
        let _ = bus.publish(&ScrollAimed { area: None });
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
