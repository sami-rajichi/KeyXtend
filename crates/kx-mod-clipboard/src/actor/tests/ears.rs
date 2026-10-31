//! The rig's ears: every event the actor publishes, kept in order.

use std::sync::{Arc, Mutex};

use kx_module_api::ports::{ClipPanel, ClipView, HistoryWanted, PasteAsk};
use kx_module_api::{Bus, Event, Notice, Subscription};

/// One event the actor published.
#[derive(Clone, Debug)]
pub(super) enum Heard {
    View(ClipView),
    Panel(ClipPanel),
    Paste(PasteAsk),
    Wanted(HistoryWanted),
    Notice(Notice),
}

/// Records every event the actor publishes into `heard`.
pub(super) fn listen(bus: &Bus, heard: &Arc<Mutex<Vec<Heard>>>) -> Vec<Subscription> {
    vec![
        ear(bus, heard, |e: &ClipView| Heard::View(e.clone())),
        ear(bus, heard, |e: &ClipPanel| Heard::Panel(*e)),
        ear(bus, heard, |e: &PasteAsk| Heard::Paste(*e)),
        ear(bus, heard, |e: &HistoryWanted| Heard::Wanted(*e)),
        ear(bus, heard, |e: &Notice| Heard::Notice(e.clone())),
    ]
}

/// Records each `E` on `bus` as `make` says.
fn ear<E: Event>(
    bus: &Bus,
    heard: &Arc<Mutex<Vec<Heard>>>,
    make: impl Fn(&E) -> Heard + Send + Sync + 'static,
) -> Subscription {
    let heard = Arc::clone(heard);
    bus.subscribe(move |e: &E| heard.lock().unwrap().push(make(e)))
        .unwrap()
}
