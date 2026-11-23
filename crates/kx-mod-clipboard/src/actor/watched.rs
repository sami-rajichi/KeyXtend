//! The watch's way into the actor (spec §7 "Limits"): copies wait in a queue of `queue_copies` while the actor keeps
//! the ones before, so an app that floods the clipboard never grows our memory. Every other note goes straight in.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, SyncSender, TrySendError};

use kx_module_api::ports::{ClipError, ClipNote, ClipRun, Clipboard};
use kx_module_api::{Clock, Mono};

use super::Msg;
use crate::settings::ClipSettings;

/// A copy the watch told, with when it read it, so a stall on the actor's thread never moves a wait's end.
pub(crate) type Stamped = (ClipNote, Mono);

/// The watch's side of the actor's inbox; it never blocks the watch's thread.
pub(crate) struct Watched {
    inbox: Sender<Msg>,
    copies: SyncSender<Stamped>,
    clock: Arc<dyn Clock>,
    flooding: AtomicBool,
}

impl Watched {
    /// The watch's side of `inbox`, with room for `room` copies stamped on `clock`, and the queue the actor takes
    /// them from.
    pub(crate) fn new(
        inbox: &Sender<Msg>,
        room: usize,
        clock: Arc<dyn Clock>,
    ) -> (Self, Receiver<Stamped>) {
        let (copies, queued) = mpsc::sync_channel(room);
        let watched = Self {
            inbox: inbox.clone(),
            copies,
            clock,
            flooding: AtomicBool::new(false),
        };
        (watched, queued)
    }

    /// Starts `clip`'s watch as `settings` say, telling `inbox` and stamping copies on `clock`; returns it and the
    /// queue of its copies.
    ///
    /// # Errors
    /// The watch's error when the OS refuses it.
    pub(crate) fn start(
        clip: &dyn Clipboard,
        settings: &ClipSettings,
        inbox: &Sender<Msg>,
        clock: &Arc<dyn Clock>,
    ) -> Result<(Box<dyn ClipRun>, Receiver<Stamped>), ClipError> {
        let (watched, copies) = Self::new(inbox, settings.queue_copies, Arc::clone(clock));
        let run = clip.watch(settings.clip_config(), Box::new(move |n| watched.tell(n)))?;
        Ok((run, copies))
    }

    /// Hands `note` to the actor, a copy stamped with the time now; a copy past the queue's room is dropped, with one
    /// warning per flood and no content.
    pub(crate) fn tell(&self, note: ClipNote) {
        if !matches!(note, ClipNote::Copied { .. }) {
            // Only a stopped actor misses it.
            let _ = self.inbox.send(Msg::Note(note));
            return;
        }
        match self.copies.try_send((note, self.clock.now())) {
            Ok(()) => {
                self.flooding.store(false, Ordering::Relaxed);
                let _ = self.inbox.send(Msg::Copied);
            }
            Err(TrySendError::Full(_)) if !self.flooding.swap(true, Ordering::Relaxed) => {
                tracing::warn!(
                    "the clipboard history fell behind a flood of copies, so new ones are dropped"
                );
            }
            Err(_) => {}
        }
    }
}
