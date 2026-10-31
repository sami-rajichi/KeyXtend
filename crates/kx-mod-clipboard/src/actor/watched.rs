//! The watch's way into the actor (spec §7 "Limits"): copies wait in a queue of `queue_copies` while the actor keeps
//! the ones before, so an app that floods the clipboard never grows our memory. Every other note goes straight in.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, SyncSender, TrySendError};

use kx_module_api::ports::{ClipError, ClipNote, ClipRun, Clipboard};

use super::Msg;
use crate::settings::ClipSettings;

/// The watch's side of the actor's inbox; it never blocks the watch's thread.
pub(crate) struct Watched {
    inbox: Sender<Msg>,
    copies: SyncSender<ClipNote>,
    flooding: AtomicBool,
}

impl Watched {
    /// The watch's side of `inbox`, with room for `room` copies, and the queue the actor takes them from.
    pub(crate) fn new(inbox: &Sender<Msg>, room: usize) -> (Self, Receiver<ClipNote>) {
        let (copies, queued) = mpsc::sync_channel(room);
        let watched = Self {
            inbox: inbox.clone(),
            copies,
            flooding: AtomicBool::new(false),
        };
        (watched, queued)
    }

    /// Starts `clip`'s watch as `settings` say, telling `inbox`; returns it and the queue of its copies.
    ///
    /// # Errors
    /// The watch's error when the OS refuses it.
    pub(crate) fn start(
        clip: &dyn Clipboard,
        settings: &ClipSettings,
        inbox: &Sender<Msg>,
    ) -> Result<(Box<dyn ClipRun>, Receiver<ClipNote>), ClipError> {
        let (watched, copies) = Self::new(inbox, settings.queue_copies);
        let run = clip.watch(settings.clip_config(), Box::new(move |n| watched.tell(n)))?;
        Ok((run, copies))
    }

    /// Hands `note` to the actor; a copy past the queue's room is dropped, with one warning per flood and no content.
    pub(crate) fn tell(&self, note: ClipNote) {
        if !matches!(note, ClipNote::Copied { .. }) {
            // Only a stopped actor misses it.
            let _ = self.inbox.send(Msg::Note(note));
            return;
        }
        match self.copies.try_send(note) {
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
