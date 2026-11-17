//! A listen-only run's speech for a cloud service: handed over only once its `Done` was told, in fresh copies each
//! call, never for a run with a model, after a Cancel, an `again`, a close or a death, and failed once on cue.

use super::*;
use kx_module_api::SecretBytes;

/// The speech a listen-only Stop hands over in the tests.
fn speech() -> Vec<Vec<u8>> {
    vec![SOUND.to_vec(), SOUND.repeat(2)]
}

/// A listen-only run, its speech set, its notes recorded.
fn listening(fake: &FakeSpeech) -> Box<dyn SpeechRun> {
    fake.set_speech(speech());
    let listen = SpeechAsk {
        model: None,
        ..ask()
    };
    fake.open(listen, Box::new(|_| {})).ok().unwrap()
}

/// The bytes of each piece `pieces` gave.
fn bytes(pieces: &[SecretBytes]) -> Vec<Vec<u8>> {
    pieces.iter().map(|p| p.expose().to_vec()).collect()
}

#[test]
fn a_listen_only_run_hands_over_its_speech_after_done_in_fresh_copies() {
    let fake = FakeSpeech::new();
    let mut run = listening(&fake);
    assert_eq!(
        run.pieces().err(),
        Some(SpeechError::NoSound),
        "before Done"
    );
    assert!(fake.tell(done(0)));
    let first = run.pieces().unwrap();
    let second = run.pieces().unwrap();
    assert_eq!((bytes(&first), bytes(&second)), (speech(), speech()));
    assert_eq!(fake.runs()[0].pieces, 2, "each call counts");
}

#[test]
fn a_run_that_heard_no_speech_hands_over_none() {
    let fake = FakeSpeech::new();
    let mut run = listening(&fake);
    fake.set_speech(Vec::new());
    assert!(fake.tell(done(0)));
    assert_eq!(run.pieces().map(|p| p.len()), Ok(0));
}

#[test]
fn no_speech_after_a_cancel_an_again_a_close_or_a_death() {
    let fake = FakeSpeech::new();
    let mut cancelled = listening(&fake);
    assert!(fake.tell(done(0)));
    assert_eq!(cancelled.cancel(), Ok(()));
    assert_eq!(cancelled.pieces().err(), Some(SpeechError::NoSound));
    let mut local = listening(&fake);
    assert!(fake.tell(done(0)));
    assert_eq!(local.again(ask_with(OTHER)), Ok(()));
    assert_eq!(local.pieces().err(), Some(SpeechError::NoSound));
    let mut closed = listening(&fake);
    assert!(fake.tell(done(0)));
    closed.close();
    assert_eq!(closed.pieces().err(), Some(SpeechError::NoSound));
    let mut dead = listening(&fake);
    assert!(fake.tell(done(0)));
    assert!(fake.die());
    assert_eq!(dead.pieces().err(), Some(SpeechError::NoSound));
}

#[test]
fn a_run_with_a_model_hands_over_no_speech() {
    let fake = FakeSpeech::new();
    fake.set_speech(speech());
    let (mut run, _heard) = opened(&fake);
    assert!(fake.tell(done(1)));
    assert_eq!(run.pieces().err(), Some(SpeechError::NoSound));
    assert_eq!(fake.runs()[0].pieces, 0, "a refused call is not counted");
}

#[test]
fn a_failure_set_for_pieces_fails_it_once() {
    let fake = FakeSpeech::new();
    let mut run = listening(&fake);
    assert!(fake.tell(done(0)));
    fake.fail(SpeechCall::Pieces, SpeechError::Internal);
    assert_eq!(run.pieces().err(), Some(SpeechError::Internal));
    assert_eq!(run.pieces().map(|p| p.len()), Ok(2));
}

#[test]
fn debug_never_prints_the_speech() {
    let fake = FakeSpeech::new();
    let _run = listening(&fake);
    let shown = format!("{fake:?} {:?}", fake.runs());
    let sound = format!("{:?}", SOUND.to_vec());
    assert!(!shown.contains(&sound[1..sound.len() - 1]), "{shown}");
}
