//! The module's actor: each click opens the dialog on a short thread of its own, so neither the actor nor a stop ever
//! waits on the dialog; it tells the state and, when the dialog does not open, the notice. It holds no state.

use crate::ID;
use kx_module_api::ports::{PowerChanged, PowerDialog, PowerError};
use kx_module_api::{Bus, Event, Notice, NoticeArg, args, keys};
use std::any::Any;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};

/// One open of the dialog, run apart from the actor.
pub type Job = Box<dyn FnOnce() + Send>;

/// Starts a job on its own thread, or says false when it cannot; tests run it in place.
pub type Spawn = fn(Job) -> bool;

/// A message for the actor, from the bus, the module or an open.
#[derive(Debug)]
pub enum Msg {
    /// The Power key was clicked.
    Asked,
    /// A module or the UI asks for the state.
    Ask,
    /// An open ended with this error.
    Failed(PowerError),
    /// An open panicked: the actor goes down with it, so the module says it is off.
    Broke(Box<dyn Any + Send>),
    /// The module stops.
    Quit,
}

/// The power module's runtime: one per start, on its own thread.
pub struct Actor {
    dialog: Arc<dyn PowerDialog>,
    bus: Bus,
    /// The actor's own inbox, where each open sends back a failure.
    back: Sender<Msg>,
    spawn: Spawn,
}

impl Actor {
    /// An actor that opens `dialog` through `spawn` and hears each open's failure on `back`, its own inbox.
    #[must_use]
    pub fn new(dialog: Arc<dyn PowerDialog>, bus: Bus, back: Sender<Msg>, spawn: Spawn) -> Self {
        Self {
            dialog,
            bus,
            back,
            spawn,
        }
    }

    /// The thread's loop: begins, then handles each message until `Quit`. It holds a sender to its own inbox, so
    /// only `Quit` ends it.
    pub fn run(self, inbox: &Receiver<Msg>) {
        self.begin();
        while let Ok(msg) = inbox.recv() {
            if !self.handle(msg) {
                return;
            }
        }
    }

    /// Says the module is live.
    pub fn begin(&self) {
        self.tell(true);
    }

    /// Handles one message; false after `Quit`, when the actor is done.
    pub fn handle(&self, msg: Msg) -> bool {
        match msg {
            Msg::Asked => self.open(),
            Msg::Ask => self.tell(true),
            Msg::Failed(error) => {
                tracing::warn!(%error, "power dialog did not open");
                self.say();
            }
            Msg::Broke(panic) => resume_unwind(panic),
            Msg::Quit => {
                self.tell(false);
                return false;
            }
        }
        true
    }

    /// Opens the dialog on its own thread. It sends back only a failure or a panic, which a stopped actor never
    /// reads, so a late answer tells nothing.
    fn open(&self) {
        let (dialog, back) = (Arc::clone(&self.dialog), self.back.clone());
        let job: Job = Box::new(move || {
            let msg = match catch_unwind(AssertUnwindSafe(|| dialog.open())) {
                Ok(Ok(())) => return,
                Ok(Err(error)) => Msg::Failed(error),
                Err(panic) => Msg::Broke(panic),
            };
            let _ = back.send(msg);
        });
        if !(self.spawn)(job) {
            tracing::warn!("power dialog's thread did not start");
            self.say();
        }
    }

    /// Tells whether the module is live.
    fn tell(&self, live: bool) {
        self.publish(&PowerChanged { live });
    }

    /// Tells the user that the dialog did not open.
    fn say(&self) {
        self.publish(&notice(keys::POWER_FAILED));
    }

    /// Publishes `event`; none of the actor's events needs a capability.
    fn publish<E: Event>(&self, event: &E) {
        let _ = self.bus.publish(event);
    }
}

/// The module's notice `key`, naming the module.
pub(crate) fn notice(key: &'static str) -> Notice {
    Notice {
        key,
        module: Some(ID),
        args: vec![(args::MODULE, NoticeArg::Text(ID.to_string()))],
    }
}
