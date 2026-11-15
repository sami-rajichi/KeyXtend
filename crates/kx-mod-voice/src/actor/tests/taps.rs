//! What the rig taps: every event the actor publishes, every text it hands the typer, and its jobs, run in place or
//! held.

use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use kx_module_api::ports::{
    LayoutAsk, PressesWanted, TextAsk, VoiceLevel, VoiceLit, VoicePage, VoiceView,
};
use kx_module_api::{Bus, Event, Notice, Subscription};

use super::super::{Job, Typist};

thread_local! {
    /// The jobs `held` keeps on this test's thread until `release`.
    static HELD: RefCell<Vec<Job>> = const { RefCell::new(Vec::new()) };
}

/// Runs a job in place, as a test has no thread to spare.
pub(super) fn in_place(job: Job) -> bool {
    job();
    true
}

/// Keeps a job for later, as a worker that takes its time.
pub(super) fn held(job: Job) -> bool {
    HELD.with(|h| h.borrow_mut().push(job));
    true
}

/// True while `held` keeps a job on this thread.
pub(super) fn waiting() -> bool {
    HELD.with(|h| !h.borrow().is_empty())
}

/// Ends every job `held` kept on this thread.
pub(super) fn release() {
    for job in HELD.with(|h| std::mem::take(&mut *h.borrow_mut())) {
        job();
    }
}

/// A typer that keeps each text in `asks`, as the keyboard would get it.
pub(super) fn typist(asks: &Arc<Mutex<Vec<TextAsk>>>) -> Typist {
    let asks = Arc::clone(asks);
    Box::new(move |ask| {
        asks.lock().unwrap().push(ask);
        true
    })
}

/// One event the actor published, as a test hears it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(
    clippy::large_enum_variant,
    reason = "a test keeps a few views, so their size does not matter"
)]
pub(super) enum Heard {
    View(VoicePage),
    Lit(bool),
    Level(u8),
    Presses(bool),
    Notice(&'static str),
    /// An ask for another part's state, by its event name.
    Ask(&'static str),
}

/// Listens on `bus` for everything the actor publishes, into `heard`.
pub(super) fn listen(bus: &Bus, heard: &Arc<Mutex<Vec<Heard>>>) -> Vec<Subscription> {
    vec![
        ear(bus, heard, |e: &VoiceView| Heard::View((*e.0).clone())),
        ear(bus, heard, |e: &VoiceLit| Heard::Lit(e.rec)),
        ear(bus, heard, |e: &VoiceLevel| Heard::Level(e.level)),
        ear(bus, heard, |e: &PressesWanted| Heard::Presses(e.on)),
        ear(bus, heard, |e: &Notice| Heard::Notice(e.key)),
        ear(bus, heard, |_: &LayoutAsk| Heard::Ask(LayoutAsk::NAME)),
    ]
}

/// Keeps each `E` on `bus` in `heard` as `make` says.
fn ear<E: Event>(
    bus: &Bus,
    heard: &Arc<Mutex<Vec<Heard>>>,
    make: impl Fn(&E) -> Heard + Send + Sync + 'static,
) -> Subscription {
    let heard = Arc::clone(heard);
    bus.subscribe(move |e: &E| heard.lock().unwrap().push(make(e)))
        .unwrap()
}
