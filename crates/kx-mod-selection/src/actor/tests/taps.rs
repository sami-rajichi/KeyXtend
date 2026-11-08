//! What the rig taps: the cap of every read the actor asks, every event it publishes, and its opens, run in place or
//! held.

use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use kx_module_api::ports::{
    AssistAsk, BoardPlaceAsk, ChipsView, EditAsk, PressesWanted, SelError, SelRun, SelSpot,
    TypingModeAsk,
};
use kx_module_api::{Bus, Event, Notice, Redacted, Subscription};

use super::super::Job;

thread_local! {
    /// The opens `held` keeps on this test's thread until `release`.
    static HELD: RefCell<Vec<Job>> = const { RefCell::new(Vec::new()) };
}

/// Runs an open in place, as a test has no thread to spare.
pub(super) fn in_place(job: Job) -> bool {
    job();
    true
}

/// Keeps an open for later, as a shell that holds the call.
pub(super) fn held(job: Job) -> bool {
    HELD.with(|h| h.borrow_mut().push(job));
    true
}

/// Ends every open `held` kept on this thread.
pub(super) fn release() {
    for job in HELD.with(|h| std::mem::take(&mut *h.borrow_mut())) {
        job();
    }
}

/// One event the actor published, as a test hears it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Heard {
    View(ChipsView),
    Edit(EditAsk),
    Presses(bool),
    Notice(Notice),
    /// An ask for another module's state, by its event name.
    Ask(&'static str),
}

/// The fake's run, keeping the box and cap of every read asked, even one the fake refuses at once in a `Closed` box.
pub(super) struct Counted {
    pub run: Box<dyn SelRun>,
    pub asked: Arc<Mutex<Vec<(u64, u16)>>>,
}

impl SelRun for Counted {
    fn look(&mut self) -> Result<SelSpot, SelError> {
        self.run.look()
    }

    fn still(&mut self, box_id: u64) -> bool {
        self.run.still(box_id)
    }

    fn read(&mut self, box_id: u64, max_chars: u16) -> Result<Redacted<String>, SelError> {
        self.asked.lock().unwrap().push((box_id, max_chars));
        self.run.read(box_id, max_chars)
    }

    fn clip_holds(&self) -> bool {
        self.run.clip_holds()
    }
}

/// Listens on `bus` for everything the actor publishes, into `heard`.
pub(super) fn listen(bus: &Bus, heard: &Arc<Mutex<Vec<Heard>>>) -> Vec<Subscription> {
    vec![
        ear(bus, heard, |e: &ChipsView| Heard::View(e.clone())),
        ear(bus, heard, |e: &EditAsk| Heard::Edit(*e)),
        ear(bus, heard, |e: &PressesWanted| Heard::Presses(e.on)),
        ear(bus, heard, |e: &Notice| Heard::Notice(e.clone())),
        ask::<BoardPlaceAsk>(bus, heard),
        ask::<AssistAsk>(bus, heard),
        ask::<TypingModeAsk>(bus, heard),
    ]
}

/// Keeps each `E`, an ask for another module's state, in `heard` by its name.
fn ask<E: Event>(bus: &Bus, heard: &Arc<Mutex<Vec<Heard>>>) -> Subscription {
    ear(bus, heard, |_: &E| Heard::Ask(E::NAME))
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
