//! What the rig hears: every event the actor publishes, kept in order.

use kx_module_api::ports::{LayoutAsk, PressesWanted, Suggested, TypeWord, TypingModeAsk};
use kx_module_api::{Bus, Event, Notice, Subscription};
use std::sync::{Arc, Mutex};

/// One event the actor published.
#[derive(Clone, Debug, PartialEq)]
pub enum Heard {
    /// A lane of chips.
    Lane(Suggested),
    /// A pick to type.
    Word(TypeWord),
    /// A notice for the user.
    Notice(Notice),
    /// Whether it wants the keyboard's presses.
    Presses(bool),
    /// An ask for the language in front.
    Ask,
    /// An ask for the typing mode.
    ModeAsk,
}

/// Hears every event the actor publishes.
pub fn listen(bus: &Bus, heard: &Arc<Mutex<Vec<Heard>>>) -> Vec<Subscription> {
    vec![
        ear(bus, heard, |e: &Suggested| Heard::Lane(e.clone())),
        ear(bus, heard, |e: &TypeWord| Heard::Word(e.clone())),
        ear(bus, heard, |e: &Notice| Heard::Notice(e.clone())),
        ear(bus, heard, |e: &PressesWanted| Heard::Presses(e.on)),
        ear(bus, heard, |_: &LayoutAsk| Heard::Ask),
        ear(bus, heard, |_: &TypingModeAsk| Heard::ModeAsk),
    ]
}

/// Keeps each `E` in `heard` as `as_heard` says.
fn ear<E: Event>(
    bus: &Bus,
    heard: &Arc<Mutex<Vec<Heard>>>,
    as_heard: fn(&E) -> Heard,
) -> Subscription {
    let heard = Arc::clone(heard);
    bus.subscribe(move |e: &E| heard.lock().unwrap().push(as_heard(e)))
        .unwrap()
}
