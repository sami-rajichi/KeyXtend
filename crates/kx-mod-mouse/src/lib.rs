//! The mouse module: Right-click by holding still, Grab, and modifier + click (spec §5.1). Its hold engine runs on
//! the platform's pointer hook (ADR-0027), so the module never talks to the OS.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod actor;
mod engine;
mod hook_engine;
mod manifest;
mod settings;

pub use actor::{Actor, Msg};
pub use engine::{ESC, Engine, EngineCfg, MAX_ACTS, MAX_LATCHES, Note};
pub use manifest::{ID, MANIFEST};
pub use settings::MouseSettings;

use kx_module_api::ports::{
    AssistAsk, AssistChanged, AssistToggle, LatchesChanged, PointerHookKey,
};
use kx_module_api::{Bus, Event, EventError, Manifest, Module, ModuleCx, ModuleError};
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};

/// The actor thread's name, as a debugger shows it.
const THREAD: &str = "kx-mouse";
/// Why the start failed when the actor's thread did not start.
const NO_THREAD: &str = "the mouse actor's thread did not start";

/// The mouse module: it reads `[mouse]` and runs its actor, which starts the hook only while needed.
#[derive(Default)]
pub struct MouseModule {
    /// The running actor's inbox and thread, and the bus it talks on, from start to stop.
    actor: Option<(Sender<Msg>, JoinHandle<()>, Bus)>,
}

impl Module for MouseModule {
    fn manifest(&self) -> &'static Manifest {
        &MANIFEST
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        self.halt();
        let settings = cx.settings::<MouseSettings>()?;
        let hook = cx.service::<PointerHookKey>()?;
        let (tx, inbox) = mpsc::channel();
        relay(cx, &tx, |e: &AssistToggle| Msg::Toggle(e.which))?;
        relay(cx, &tx, |_: &AssistAsk| Msg::Ask)?;
        relay(cx, &tx, |e: &LatchesChanged| Msg::Latches(e.held.clone()))?;
        let bus = cx.bus();
        let actor = Actor::new(
            hook,
            bus.clone(),
            cx.settings_handle(),
            settings,
            tx.clone(),
        );
        let thread = thread::Builder::new()
            .name(THREAD.to_owned())
            .spawn(move || actor.run(&inbox))
            .map_err(|_| ModuleError::Start(NO_THREAD))?;
        self.actor = Some((tx, thread, bus));
        Ok(())
    }

    /// Joins the actor, so its hook lets go of anything held before the module is gone.
    fn stop(&mut self) {
        self.halt();
    }
}

impl Drop for MouseModule {
    fn drop(&mut self) {
        self.halt();
    }
}

impl MouseModule {
    /// Tells the actor to stop and waits for its thread to end. A panic there is logged, with no point or key, and the
    /// module says it is not live, as the actor could not, so the keyboard dims its keys.
    fn halt(&mut self) {
        if let Some((tx, thread, bus)) = self.actor.take() {
            let _ = tx.send(Msg::Stop);
            if thread.join().is_err() {
                tracing::warn!("mouse actor panicked");
                // `AssistChanged` needs no capability, so this cannot be refused.
                let _ = bus.publish(&AssistChanged::OFF);
            }
        }
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
