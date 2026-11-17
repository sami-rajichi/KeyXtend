//! The voice job's loop over in-memory pipes: asks answered in order until goodbye, a clean end, a broken frame, a
//! panic told as an internal fault on each run it touched, and a note that fails its own checks sent as one.

use super::*;
use crate::voice::fake::{self, Got};
use crate::voice::mic::{Format, Opened, Source};
use kx_ipc::{MAX_PIECES, Model, Setup, Stage, Target};
use std::borrow::Cow;
use std::cell::RefCell;
use std::io::{Cursor, ErrorKind};
use std::rc::Rc;
use zeroize::Zeroizing;

/// A source that panics as it opens.
struct Boom;

impl Source for Boom {
    fn open(&mut self, _: Feed) -> Result<Opened, VoiceFault> {
        panic!("the test source panics");
    }
}

/// A source whose opening panics as it closes.
struct Sticky;

/// The open `Sticky` source.
struct Stuck;

impl Drop for Stuck {
    fn drop(&mut self) {
        panic!("the test source panics as it closes");
    }
}

impl Source for Sticky {
    fn open(&mut self, _: Feed) -> Result<Opened, VoiceFault> {
        let format = Format {
            rate: 16_000,
            channels: 1,
        };
        let live = Box::new(Stuck);
        Ok(Opened { format, live })
    }
}

/// Input whose every read panics.
struct Bomb;

impl Read for Bomb {
    fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
        panic!("the test input panics");
    }
}

/// Notes that panic on a piece's text, as a failing writer might.
#[derive(Default)]
struct Fussy(Vec<VoiceNote>);

impl Notes for Fussy {
    fn tell(&mut self, note: VoiceNote) -> Result<(), IpcError> {
        let piece = matches!(note, VoiceNote::Piece { .. });
        assert!(!piece, "the test's notes panic on a piece");
        self.0.push(note);
        Ok(())
    }
}

/// The fake parts over a closed 16 kHz mono source.
fn parts() -> Parts {
    fake::parts(
        &fake::mic(16_000, 1),
        false,
        &Rc::<RefCell<Vec<Got>>>::default(),
    )
}

/// A setup at the bottom of each range, with `model` or none.
fn setup(model: Option<Model<'static>>) -> Setup<'static> {
    Setup {
        model,
        pad_ms: 0,
        piece_s: *kx_ipc::PIECE_S.start(),
        record_max_s: *kx_ipc::RECORD_MAX_S.start(),
        level_hz: *kx_ipc::LEVEL_HZ.start(),
        silence_stop_s: None,
        detector: Cow::Borrowed("detector.onnx"),
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

fn frames(asks: &[VoiceAsk<'_>]) -> Vec<u8> {
    let mut out = Vec::new();
    for ask in asks {
        ask.write(&mut out).unwrap();
    }
    out
}

fn notes(mut out: &[u8]) -> Vec<VoiceNote> {
    std::iter::from_fn(|| VoiceNote::read(&mut out).ok()).collect()
}

/// Runs the loop over `input`; its outcome and every note it wrote.
fn serve_over(input: Vec<u8>, parts: Parts) -> (Result<(), IpcError>, Vec<VoiceNote>) {
    let mut out = Vec::new();
    let outcome = run(Cursor::new(input), &mut out, parts);
    (outcome, notes(&out))
}

fn fault(id: u32, fault: VoiceFault) -> VoiceNote {
    VoiceNote::Fault { id, fault }
}

fn open(id: u32) -> VoiceNote {
    let stage = Stage::Open;
    VoiceNote::Stage { id, stage }
}

#[test]
fn asks_are_answered_in_order_until_goodbye() {
    let asks = [
        VoiceAsk::Keep { id: 1 },
        VoiceAsk::Devices { id: 2 },
        VoiceAsk::Close { id: 1 },
        VoiceAsk::Bye,
        VoiceAsk::Keep { id: 3 },
    ];
    let (outcome, told) = serve_over(frames(&asks), parts());
    assert_eq!(outcome, Ok(()));
    let shut = VoiceNote::Stage {
        id: 1,
        stage: Stage::Shut,
    };
    assert_eq!(
        told,
        [
            fault(1, VoiceFault::NoSound),
            fault(2, VoiceFault::Runner),
            shut
        ]
    );
}

#[test]
fn the_end_of_input_ends_cleanly_and_a_broken_frame_with_its_error() {
    assert_eq!(serve_over(Vec::new(), parts()), (Ok(()), Vec::new()));
    let (outcome, told) = serve_over(b"XXXX not a frame".to_vec(), parts());
    assert_eq!((outcome, told), (Err(IpcError::BadMagic), Vec::new()));
}

#[test]
fn a_read_that_panics_ends_the_job_as_a_failed_stream() {
    let mut out = Vec::new();
    let outcome = run(Bomb, &mut out, parts());
    assert_eq!(outcome, Err(IpcError::Io(ErrorKind::Other)));
    assert!(out.is_empty());
}

#[test]
fn a_panic_wipes_the_job_and_is_told_as_an_internal_fault() {
    let mut parts = parts();
    parts.source = Box::new(Boom);
    let start = VoiceAsk::Start {
        id: 1,
        setup: setup(None),
    };
    let asks = [start, VoiceAsk::Keep { id: 2 }];
    let (outcome, told) = serve_over(frames(&asks), parts);
    assert_eq!(outcome, Ok(()));
    assert_eq!(
        told,
        [
            fault(1, VoiceFault::Internal),
            fault(2, VoiceFault::NoSound)
        ]
    );
}

#[test]
fn a_panic_on_an_engine_note_closes_the_listening_run_with_a_fault() {
    let (mic, log) = (fake::mic(16_000, 1), Rc::default());
    let (tx, _rx) = mpsc::channel();
    let (mut job, mut out) = (
        Job::new(fake::parts(&mic, true, &log), tx),
        Fussy::default(),
    );
    let setup = setup(Some(model()));
    step(
        &mut job,
        Event::Ask(VoiceAsk::Start { id: 1, setup }),
        &mut out,
    )
    .unwrap();
    let text = Zeroizing::new(String::new());
    let note = VoiceNote::Piece {
        id: 1,
        index: 0,
        text,
    };
    let flow = step(&mut job, Event::Note(note), &mut out);
    assert!(matches!(flow, Ok(Flow::Go)));
    assert_eq!(out.0, [open(1), fault(1, VoiceFault::Internal)]);
    assert!(!mic.borrow().open);
}

#[test]
fn a_panic_at_goodbye_or_the_end_of_input_still_ends_the_job() {
    let short = Err(IpcError::Short);
    for (end, want) in [
        (Event::Ask(VoiceAsk::Bye), Ok(())),
        (Event::End(short), short),
    ] {
        let mut parts = parts();
        parts.source = Box::new(Sticky);
        let (tx, _rx) = mpsc::channel();
        let (mut job, mut out) = (Job::new(parts, tx), Vec::new());
        let start = VoiceAsk::Start {
            id: 1,
            setup: setup(None),
        };
        step(&mut job, Event::Ask(start), &mut out).unwrap();
        let flow = step(&mut job, end, &mut out).unwrap();
        assert!(matches!(flow, Flow::Exit(got) if got == want));
        assert_eq!(out, [open(1), fault(1, VoiceFault::Internal)]);
    }
}

#[test]
fn a_note_that_fails_its_own_checks_goes_as_an_internal_fault() {
    let mut out = Vec::new();
    let index = u16::try_from(MAX_PIECES).unwrap();
    let text = Zeroizing::new(String::new());
    Pipe(&mut out)
        .tell(VoiceNote::Piece { id: 4, index, text })
        .unwrap();
    assert_eq!(notes(&out), [fault(4, VoiceFault::Internal)]);
}
