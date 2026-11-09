//! Word prediction on the face (ADR-0033): the typed pieces leave in order, one thread at a time and outside the lock,
//! so the bus hears each once; a picked word from the bus types on the sender's thread, one part per lock, tells each
//! part's pieces and waits its pauses outside the lock, and stops at a click outside our windows or a board key sent.

use super::Face;
use super::inner::{Queued, WordTo};
use crate::run::{Word, drive};
use kx_module_api::ports::{AimedLine, PanelKey, PointerPressed, TypeWord, Typed};
use std::sync::atomic::Ordering::SeqCst;

impl Face {
    /// Tells the bus every piece typed, line change and edit answer not yet told, through the outbox, so none goes out
    /// twice or out of order.
    pub(super) fn pour(&self) {
        self.pieces.post(|| self.send_pieces());
    }

    /// Sends what waits, oldest first, until none waits; only the sending thread calls it.
    fn send_pieces(&self) {
        loop {
            let (told, bus) = {
                let mut inner = self.lock();
                (inner.take_told(), inner.bus())
            };
            if told.is_empty() {
                return;
            }
            for item in told {
                // The module holds `ReadFocusedText` and `Selection`, which these events need, or its start would have
                // failed.
                let _ = match item {
                    Queued::Piece(piece) => bus.publish(&Typed { piece }),
                    Queued::Line(line) => bus.publish(&AimedLine { line }),
                    Queued::Done(done) => bus.publish(&done),
                };
            }
        }
    }

    /// A picked word from the prediction module, whose round that module checked, goes where its target and the aim
    /// agree (ADR-0040 decision 8), or nowhere; then the typing mode goes out.
    pub(super) fn type_word(&self, word: &TypeWord) {
        let seen = self.clicks.load(SeqCst);
        let to = self.lock().word_to(word, seen);
        match to {
            WordTo::App(left) => self.drive_word(left, seen),
            WordTo::Line(panel) => self.key_to(
                panel,
                PanelKey::Word {
                    erase: word.erase,
                    text: word.text.clone(),
                },
            ),
            WordTo::Nowhere => {}
        }
        self.pour();
        self.tell(false);
    }

    /// Types `left` as keys would, telling its pieces after each part and waiting each pause on the sender's thread with
    /// the lock free. A click outside our windows since `seen`, or a board key sent meanwhile, stops the rest.
    fn drive_word(&self, mut left: Word, seen: u64) {
        let pause = self.lock().pause();
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

    /// A press the mouse module heard: one outside our windows may have moved the caret, so a word going out stops and
    /// the aim goes when next read. It takes no lock, as the sender may be mid-word.
    pub(super) fn pressed(&self, press: PointerPressed) {
        if !press.ours {
            self.clicks.fetch_add(1, SeqCst);
        }
    }

    /// The module started, or stopped when `on` is false; a picked word, a paste and a fill go only while it runs, and
    /// presses are wanted only then.
    pub(crate) fn set_running(&self, on: bool) {
        if on {
            self.lock().set_running(true);
        } else {
            self.fill_stop();
        }
        self.pour();
        self.tell(false);
    }
}
