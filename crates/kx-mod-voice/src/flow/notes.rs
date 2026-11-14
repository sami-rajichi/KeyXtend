//! What the run tells: the microphone's stages, the level, speech heard, the pieces, the end, the recording wiped and
//! the faults. A note of a closed or older dictation is dropped.

use kx_module_api::Secret;
use kx_module_api::ports::{
    BarButton, BarHint, SpeechDone, SpeechError, SpeechNote, SpeechStage, VoiceLevel,
};

use super::take::Lang;
use super::{Count, Flow, Out, Phase};

impl Flow {
    /// A note from dictation `id`'s run.
    pub(super) fn note(&mut self, id: u32, note: SpeechNote, out: &mut Vec<Out>) {
        let live = self.take.as_ref().is_some_and(|t| t.id == id && t.open);
        if !live {
            return;
        }
        match note {
            SpeechNote::Stage(stage) => self.stage(stage, out),
            SpeechNote::Level(level) => self.level(level, out),
            SpeechNote::Heard => self.heard(),
            SpeechNote::Quiet | SpeechNote::Warming { .. } => {}
            SpeechNote::Piece { index, text } => self.piece(index, text),
            SpeechNote::Done(done) => self.done(done, out),
            SpeechNote::Wiped => self.lose(),
            SpeechNote::Fault(error) => self.fault(error, out),
        }
    }

    /// The light follows the microphone; the job stopping by itself ends listening.
    fn stage(&mut self, stage: SpeechStage, out: &mut Vec<Out>) {
        let listening = self.phase() == Phase::Listening;
        match stage {
            SpeechStage::Loading => {}
            SpeechStage::Open => self.lit = listening,
            SpeechStage::Shut => self.lit = false,
            SpeechStage::Limit | SpeechStage::Silence => {
                if let Some(take) = self.take.as_mut() {
                    take.long |= stage == SpeechStage::Limit;
                }
                self.lit = false;
                if listening {
                    self.stop(false, out);
                }
            }
        }
    }

    /// The level reaches the bar only while listening.
    fn level(&self, level: u8, out: &mut Vec<Out>) {
        if self.phase() == Phase::Listening {
            out.push(Out::Level(VoiceLevel { level }));
        }
    }

    /// Speech heard clears the nothing-heard hint.
    fn heard(&mut self) {
        if let Some(take) = self.take.as_mut() {
            take.heard = true;
        }
        if let super::State::Listening { nothing, .. } = &mut self.state {
            *nothing = false;
        }
    }

    /// A piece is kept, unshown, while the run writes.
    fn piece(&mut self, index: u16, text: Secret) {
        let writing = self.busy();
        if let Some(take) = self.take.as_mut().filter(|_| writing) {
            take.pieces.insert(index, text);
        }
    }

    /// The run wrote every piece: the text goes out; with none, Try again's bar comes back, else nothing was heard.
    fn done(&mut self, done: SpeechDone, out: &mut Vec<Out>) {
        if !self.busy() {
            return;
        }
        self.lit = false;
        let Some(take) = self.take.as_mut() else {
            return;
        };
        if !self.hidden {
            let model = take.ready.name.clone();
            let card = take.ask.card.as_ref().filter(|_| done.card);
            let card = card.map(|c| c.id.clone());
            out.push(Out::Count(Count { model, card, done }));
        }
        let before = take.before.take();
        let text = take.joined();
        if !text.is_empty() {
            return self.deliver(text, out);
        }
        match before {
            Some(before) => {
                self.state = before;
                self.shown_at = self.now();
            }
            None => self.nothing(false, out),
        }
    }

    /// The recording is gone, so Try again and Transcribe anyway dim.
    fn lose(&mut self) {
        if let Some(take) = self.take.as_mut() {
            take.held = false;
        }
    }

    /// A fault: a card's lets the processor carry on; another ends a busy run; after the text, a death loses the
    /// recording.
    fn fault(&mut self, error: SpeechError, out: &mut Vec<Out>) {
        if !error.ends() {
            return self.fell(error, out);
        }
        if self.busy() {
            return self.broke(error, out);
        }
        if matches!(
            error,
            SpeechError::Gone | SpeechError::Timeout | SpeechError::NoSound
        ) {
            self.lose();
        }
    }

    /// A card's fault: the processor writes the same recording, and the failed card's check of this model is dropped.
    fn fell(&mut self, error: SpeechError, out: &mut Vec<Out>) {
        let Some(take) = self.take.as_mut() else {
            return;
        };
        take.fell = true;
        let card = take.ask.card.as_ref().map(|c| c.id.clone());
        if let (SpeechError::CardFailed, Some(card)) = (error, card) {
            let model = take.ready.print;
            out.push(Out::Fell { card, model });
        }
    }

    /// A fault ended a busy run: the pieces written stay with Copy under the fault's line, else its note shows.
    fn broke(&mut self, error: SpeechError, out: &mut Vec<Out>) {
        let Some(take) = &self.take else {
            return;
        };
        let pieces = take.joined();
        let (lang, bytes) = (take.lang.clone(), take.ready.bytes);
        self.end_run(false, out);
        if error == SpeechError::NoFile {
            out.push(Out::Lost);
        }
        let (hint, act) = fault_note(error, &lang, bytes);
        if !pieces.is_empty() {
            let death = matches!(error, SpeechError::Gone | SpeechError::Timeout);
            return self.salvage(pieces, if death { BarHint::Pieces } else { hint }, out);
        }
        self.take = None;
        self.show_note(hint, act, lang.iso);
    }
}

/// The note a fault that ended the run shows, with its action: Listen again for the microphone, Download for a file
/// gone, the gear for the language.
fn fault_note(error: SpeechError, lang: &Lang, bytes: u64) -> (BarHint, Option<BarButton>) {
    let listen = Some(BarButton::Listen);
    let shown = lang.shown.clone();
    match error {
        SpeechError::NoMic => (BarHint::NoMic, listen),
        SpeechError::MicBusy => (BarHint::MicBusy, listen),
        SpeechError::MicRefused => (BarHint::MicRefused, listen),
        SpeechError::Unplugged => (BarHint::Unplugged, listen),
        SpeechError::NoFile => (
            BarHint::NoModel { lang: shown, bytes },
            Some(BarButton::Download),
        ),
        SpeechError::Language => (BarHint::Language(shown), None),
        SpeechError::Memory => (BarHint::NoFit, None),
        _ => (BarHint::Fault, None),
    }
}
