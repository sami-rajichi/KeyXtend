//! The rig's ears: every event the actor publishes, kept in order.

use std::sync::{Arc, Mutex};

use kx_module_api::ports::{
    ClipTake, FillLine, FillLive, FillPanel, FillType, FillView, PanelLine, VaultEmails,
};
use kx_module_api::{Bus, Event, Notice, Subscription};

/// One event the actor published.
#[derive(Clone, Debug)]
pub(super) enum Heard {
    Live(FillLive),
    Panel(FillPanel),
    Line(PanelLine),
    View(FillView),
    Type(FillType),
    Emails(VaultEmails),
    Take(ClipTake),
    Notice(Notice),
}

/// Records every event the actor publishes into `heard`.
pub(super) fn listen(bus: &Bus, heard: &Arc<Mutex<Vec<Heard>>>) -> Vec<Subscription> {
    vec![
        ear(bus, heard, |e: &FillLive| Heard::Live(*e)),
        ear(bus, heard, |e: &FillPanel| Heard::Panel(*e)),
        ear(bus, heard, |e: &FillLine| Heard::Line(e.0.clone())),
        ear(bus, heard, |e: &FillView| Heard::View(e.clone())),
        ear(bus, heard, |e: &FillType| Heard::Type(e.clone())),
        ear(bus, heard, |e: &VaultEmails| Heard::Emails(e.clone())),
        ear(bus, heard, |e: &ClipTake| Heard::Take(*e)),
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
