//! Word prediction on the face (ADR-0033): the typed pieces leave in order, one thread at a time and outside the lock,
//! so the bus hears each once; a picked word from the bus types on the sender's thread, one part per lock, tells each
//! part's pieces and waits its pauses outside the lock, and stops at a click outside our windows or a board key sent.

use super::Face;
use crate::run::drive;
use kx_module_api::ports::{PointerPressed, TypeWord, Typed};
use std::sync::atomic::Ordering::SeqCst;

impl Face {
    /// Tells the bus every piece typed and not yet told, through the outbox, so none goes out twice or out of order.
    pub(super) fn pour(&self) {
        self.pieces.post(|| self.send_pieces());
    }

    /// Sends the waiting pieces, oldest first, until none waits; only the sending thread calls it.
    fn send_pieces(&self) {
        loop {
            let (pieces, bus) = {
                let mut inner = self.lock();
                (inner.take_pieces(), inner.bus())
            };
            if pieces.is_empty() {
                return;
            }
            for piece in pieces {
                // The module holds `ReadFocusedText`, which this event needs, or its start would have failed.
                let _ = bus.publish(&Typed { piece });
            }
        }
    }

    /// A picked word from the prediction module, whose round that module checked: it types as keys would, telling its
    /// pieces after each part and waiting each pause on the sender's thread with the lock free. A click outside our
    /// windows, or a board key sent meanwhile, stops the rest; then the typing mode goes out.
    pub(super) fn type_word(&self, word: &TypeWord) {
        let seen = self.clicks.load(SeqCst);
        let (start, pause) = {
            let inner = self.lock();
            (inner.start_word(word.erase, &word.text), inner.pause())
        };
        if let Some(mut left) = start {
            let away = || self.clicks.load(SeqCst) != seen;
            drive(
                || {
                    let part = self.lock().word_part(&mut left, away());
                    self.pour();
                    part
                },
                &*pause,
            );
        }
        self.pour();
        self.tell(false);
    }

    /// A press the mouse module heard: one outside our windows may have moved the caret, so a word going out stops.
    /// It takes no lock, as the sender may be mid-word.
    pub(super) fn pressed(&self, press: PointerPressed) {
        if !press.ours {
            self.clicks.fetch_add(1, SeqCst);
        }
    }

    /// The module started, or stopped when `on` is false; a picked word types and a paste goes only while it runs.
    pub(crate) fn set_running(&self, on: bool) {
        self.lock().set_running(on);
    }
}
