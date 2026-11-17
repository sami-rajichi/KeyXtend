//! Voice typing on the face (ADR-0042 decision 7): the Dictation key asks the voice module to start or stop, its light
//! follows `VoiceLit`, and a text from the bus types on the sender's thread, one piece per lock, waiting its gap with
//! the lock free. A click outside our windows, or a key, word or text sent, stops the rest. As in `bus.rs`, nothing
//! is sent while the face's lock is held, and no text is ever logged.

use super::{Face, STATE};
use crate::run::{Text, drive};
use kx_module_api::ports::{FaceReply, TextAsk, VoiceKey, VoiceLit};
use std::sync::atomic::Ordering::SeqCst;

impl Face {
    /// The Dictation key: asks the voice module to start or stop, keeping every latch. The state redraws, so a light
    /// the module tells at once shows at once.
    pub(super) fn voice_key(&self) -> FaceReply {
        let bus = self.lock().bus();
        // The module holds `Voice`, which this event needs, or its start would have failed.
        let _ = bus.publish(&VoiceKey);
        STATE
    }

    /// The voice module says whether the microphone is open.
    pub(super) fn heard_lit(&self, lit: VoiceLit) {
        self.lock().heard_voice(lit);
    }

    /// The voice module asks for text `ask`: it erases, then types, telling its pieces after each part. Then its answer
    /// leaves through the pieces' sender, after its own break, and the typing mode goes out.
    pub(super) fn type_text(&self, ask: &TextAsk) {
        let seen = self.clicks.load(SeqCst);
        let started = self.lock().start_text(ask);
        if let Some(mut text) = started {
            self.drive_text(&mut text, seen);
            self.lock().text_done(text.done());
        }
        self.pour();
        self.tell(false);
    }

    /// Sends `text` one piece per lock, waiting each gap on the sender's thread with the lock free. A click outside our
    /// windows since `seen` stops the rest; a key, word or text sent meanwhile stops it in `Typing::text_part`.
    fn drive_text(&self, text: &mut Text, seen: u64) {
        let pause = self.lock().pause();
        let away = || self.clicks.load(SeqCst) != seen;
        drive(
            || {
                let part = self.lock().text_part(text, away());
                self.pour();
                part
            },
            &*pause,
        );
    }
}
