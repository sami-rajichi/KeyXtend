//! A run's timer: a thread of its own that waits until its run's due time on the run's clock, then asks for a check;
//! a new due time or a poke wakes it, and it ends with its run. It never tells a note itself.

use kx_module_api::{Clock, Mono};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;

/// The timer's thread.
const THREAD: &str = "kx-net-time";

/// What wakes a timer.
pub(super) enum Wake {
    /// The run's due time is now this, or none.
    At(Option<Mono>),
    /// Look at the clock again, as a test that moved it asks, then answer once any check is asked for.
    #[cfg(test)]
    Poke(Sender<()>),
}

/// Starts a timer on `clock` that calls `fire` each time its due time passes, until `fire` says its run is gone or
/// every sender of its wakes is dropped; `None` when its thread does not start.
pub(super) fn start(
    clock: Arc<dyn Clock>,
    fire: impl Fn() -> bool + Send + 'static,
) -> Option<Sender<Wake>> {
    let (wake, woken) = mpsc::channel();
    let body = move || watch(clock.as_ref(), &woken, &fire);
    let started = thread::Builder::new().name(THREAD.to_owned()).spawn(body);
    started
        .inspect_err(|e| tracing::warn!(error = %e, "a cloud timer did not start"))
        .ok()?;
    Some(wake)
}

/// Waits for each due time in turn and fires it once.
fn watch(clock: &dyn Clock, woken: &Receiver<Wake>, fire: &dyn Fn() -> bool) {
    let mut due = None;
    loop {
        let got = match due {
            Some(at) => woken.recv_timeout(Mono::saturating_since(at, clock.now())),
            None => woken.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };
        match got {
            Ok(Wake::At(at)) => due = at,
            #[cfg(test)]
            Ok(Wake::Poke(looked)) => {
                let live = check(clock, &mut due, fire);
                let _ = looked.send(());
                if !live {
                    return;
                }
                continue;
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
        if !check(clock, &mut due, fire) {
            return;
        }
    }
}

/// Fires once `due` passed and clears it; false once `fire` says the run is gone.
fn check(clock: &dyn Clock, due: &mut Option<Mono>, fire: &dyn Fn() -> bool) -> bool {
    if due.is_some_and(|at| at <= clock.now()) {
        *due = None;
        return fire();
    }
    true
}
