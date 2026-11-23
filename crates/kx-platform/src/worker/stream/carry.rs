//! A stream's readers and their hand-over: each reader counts a frame as carried before it stamps it, and a wait that
//! ends while a frame is carried waits for that frame, as it was stamped in time.

use super::{Event, IN_FLIGHT, Spawner, Split, Streams};
use kx_ipc::IpcError;
use kx_module_api::Clock;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

/// The next event within `wait`, if any; past it, while a reader still carries a frame (`carried`), that frame is
/// waited for, as it was stamped before the wait ended.
pub(super) fn next<E>(
    inbox: &Receiver<E>,
    wait: Option<Duration>,
    carried: &AtomicUsize,
) -> Result<E, RecvTimeoutError> {
    if let Some(wait) = wait {
        match inbox.recv_timeout(wait) {
            Err(RecvTimeoutError::Timeout) if carried.load(SeqCst) > 0 => {}
            other => return other,
        }
    }
    inbox.recv().map_err(|_| RecvTimeoutError::Disconnected)
}

/// A reader thread: it counts each frame of worker `nth` as carried, stamps it and hands it to the stream, with
/// `IN_FLIGHT` untaken at most, until one fails or the stream is gone.
pub(super) fn relay<S: Spawner<F, Worker: Split<F>>, F: Streams>(
    nth: u64,
    notes: &Receiver<Result<F::Reply, IpcError>>,
    feed: &Sender<Event<S, F>>,
    clock: &dyn Clock,
    carried: &AtomicUsize,
) {
    let mut untaken = VecDeque::with_capacity(IN_FLIGHT);
    loop {
        if untaken.len() >= IN_FLIGHT {
            // Ends once the stream took the oldest frame, or dropped it with the stream.
            let _ = untaken
                .pop_front()
                .map(|oldest: Receiver<()>| oldest.recv());
        }
        let frame = notes.recv().unwrap_or(Err(IpcError::Closed));
        carried.fetch_add(1, SeqCst);
        let (last, at, (taken, room)) = (frame.is_err(), clock.now(), mpsc::channel());
        let heard = Event::Heard {
            nth,
            frame,
            at,
            taken,
        };
        if feed.send(heard).is_err() || last {
            return;
        }
        untaken.push_back(room);
    }
}
