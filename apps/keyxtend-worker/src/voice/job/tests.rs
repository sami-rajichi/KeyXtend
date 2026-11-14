//! The job driven by hand: a fake source the test feeds, a detector that hears loud frames and an engine that logs
//! its pieces. Events are pumped on the test's thread, so nothing sleeps and no microphone opens.

mod again;
mod asks;
mod engine;
mod hand;
mod listen;

use super::*;
use crate::voice::fake::{self, Got, LOUD, Mic};
use crate::voice::wipe::Buf;
use kx_ipc::{Model, PIECE_S, SAMPLE_BYTES, SOUND_HZ, SoundPiece, Target};
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver};

/// The level rate the tests ask for.
const HZ: u8 = 10;
/// The sound rate the tests' source gives.
const RATE: u32 = 16_000;
/// Samples in a second of the job's sound.
const SEC: usize = SOUND_HZ;
/// The samples a chunk of the fake source holds, 10 ms.
const CHUNK: usize = SEC / 100;

/// The tests' list of notes, checked as the pipe would check them.
impl Notes for Vec<VoiceNote> {
    fn tell(&mut self, note: VoiceNote) -> Result<(), IpcError> {
        note.check()?;
        self.push(note);
        Ok(())
    }
}

/// A job over fake parts, and what the test sees of them.
struct Rig {
    job: Job,
    tx: Sender<Event>,
    rx: Receiver<Event>,
    notes: Vec<VoiceNote>,
    mic: Rc<RefCell<Mic>>,
    log: Rc<RefCell<Vec<Got>>>,
}

impl Rig {
    /// A job whose source gives `rate` frames of `channels`, with a detector when `detector`.
    fn new(rate: u32, channels: u16, detector: bool) -> Self {
        let (mic, log) = (fake::mic(rate, channels), Rc::default());
        let parts = fake::parts(&mic, detector, &log);
        Self::over(mic, log, parts)
    }

    /// A job over `parts`, whose source is `mic` and whose fake engine, if any, logs to `log`.
    fn over(mic: Rc<RefCell<Mic>>, log: Rc<RefCell<Vec<Got>>>, parts: Parts) -> Self {
        let (tx, rx) = mpsc::channel();
        let job = Job::new(parts, tx.clone());
        let notes = Vec::new();
        Self {
            job,
            tx,
            rx,
            notes,
            mic,
            log,
        }
    }

    /// A job over a 16 kHz mono source without a detector.
    fn plain() -> Self {
        Self::new(RATE, 1, false)
    }

    /// Takes `ask` and the events it caused; the notes told.
    fn ask(&mut self, ask: VoiceAsk<'static>) -> Vec<VoiceNote> {
        self.only(ask);
        self.pump();
        self.told()
    }

    /// Takes `ask` alone, leaving the events it caused queued.
    fn only(&mut self, ask: VoiceAsk<'static>) {
        let flow = self.job.take(Event::Ask(ask), &mut self.notes).unwrap();
        assert!(matches!(flow, Flow::Go));
    }

    /// Feeds `x` to the last opening of the source, a chunk at a time, taking each; the notes told.
    fn speak(&mut self, x: &[f32], chunk: usize) -> Vec<VoiceNote> {
        let feed = self.mic.borrow().feed.clone().unwrap();
        for c in x.chunks(chunk) {
            feed.sound(Buf::collect(c.iter().copied()).unwrap());
            self.pump();
        }
        self.told()
    }

    /// Takes every queued event.
    fn pump(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            let flow = self.job.take(event, &mut self.notes).unwrap();
            assert!(matches!(flow, Flow::Go));
        }
    }

    /// The notes told since the last call.
    fn told(&mut self) -> Vec<VoiceNote> {
        std::mem::take(&mut self.notes)
    }

    /// True while the source is open.
    fn open(&self) -> bool {
        self.mic.borrow().open
    }
}

/// A model on the processor.
fn model() -> Model<'static> {
    Model {
        path: Cow::Borrowed("model.gguf"),
        lang: Cow::Borrowed("en"),
        runner: Cow::Borrowed("transcribe"),
        target: Target::Cpu,
        threads: 0,
    }
}

/// A setup with a model or none, pieces of 10 s, a 300 ms pad, a 600 s limit and no silence stop.
fn setup(with_model: bool) -> Setup<'static> {
    Setup {
        model: with_model.then(model),
        pad_ms: 300,
        piece_s: *PIECE_S.start(),
        record_max_s: 600,
        level_hz: HZ,
        silence_stop_s: None,
        detector: Cow::Borrowed("detector.onnx"),
    }
}

/// Starts run `id` listening only.
fn start(id: u32) -> VoiceAsk<'static> {
    VoiceAsk::Start {
        id,
        setup: setup(false),
    }
}

/// `n` samples of speech, as floats that make `LOUD` exactly.
fn loud(n: usize) -> Vec<f32> {
    vec![f32::from(LOUD) / f32::from(i16::MAX); n]
}

/// `n` samples of silence.
fn silent(n: usize) -> Vec<f32> {
    vec![0.0; n]
}

/// `n` quiet samples counting up from 0 to 199, as floats that make those numbers exactly.
fn ramp(n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| f32::from(u8::try_from(i % 200).unwrap()) / f32::from(i16::MAX))
        .collect()
}

/// The 16-bit samples `ramp(n)` records.
fn ramp16(n: usize) -> Vec<i16> {
    (0..n).map(|i| i16::try_from(i % 200).unwrap()).collect()
}

/// The samples `notes` hand over, and each `Sound` note's index, last flag and length.
fn sounds(notes: &[VoiceNote]) -> (Vec<i16>, Vec<(u16, bool, usize)>) {
    let (mut samples, mut pieces) = (Vec::new(), Vec::new());
    for note in notes {
        if let VoiceNote::Sound { piece, .. } = note {
            pieces.push((piece.index, piece.last, piece.bytes.len()));
            let pairs = piece.bytes.as_chunks::<SAMPLE_BYTES>().0;
            samples.extend(pairs.iter().map(|b| i16::from_le_bytes(*b)));
        }
    }
    (samples, pieces)
}

/// The levels among `notes`.
fn levels(notes: &[VoiceNote]) -> Vec<u8> {
    let level = |n: &VoiceNote| match n {
        VoiceNote::Level { level, .. } => Some(*level),
        _ => None,
    };
    notes.iter().filter_map(level).collect()
}

/// `notes` without levels and speech heard or lost.
fn plain(notes: &[VoiceNote]) -> Vec<VoiceNote> {
    let shown = |n: &&VoiceNote| !matches!(n, VoiceNote::Level { .. } | VoiceNote::Heard { .. });
    notes.iter().filter(shown).cloned().collect()
}

/// A fault note.
fn fault(id: u32, fault: VoiceFault) -> VoiceNote {
    VoiceNote::Fault { id, fault }
}

/// A piece note as the fake writer tells it.
fn piece(id: u32, index: u16) -> VoiceNote {
    let text = zeroize::Zeroizing::new(fake::WORDS.to_owned());
    VoiceNote::Piece { id, index, text }
}

/// A `Sound` ask handing in piece `index` of `samples` for run `id`.
fn hand_in(id: u32, index: u16, last: bool, samples: &[i16]) -> VoiceAsk<'static> {
    let bytes: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
    let bytes = kx_ipc::Wiped::Kept(zeroize::Zeroizing::new(bytes));
    let piece = SoundPiece { index, last, bytes };
    VoiceAsk::Sound { id, piece }
}
