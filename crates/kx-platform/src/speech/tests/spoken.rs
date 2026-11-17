//! A cloud dictation listens on the `voice` job with no model and the row's `piece_s`; its speech comes back at Stop as
//! wiped pieces once `Done` is told, again for Try again, and never before Stop, after a Cancel, a close or a wipe.

use super::*;
use crate::worker::Told;
use crate::worker::fake::voice::{open as opened, shut};
use kx_ipc::MAX_SPEECH_PIECES;
use proptest::prelude::*;

/// The longest piece a test row allows, in seconds.
const ROW_S: u8 = 25;

/// A piece of run `id`'s speech.
fn spoken(id: u32, index: u16, last: bool) -> VoiceNote {
    let bytes = Wiped::Kept(Zeroizing::new(SOUND.to_vec()));
    let piece = SoundPiece { index, last, bytes };
    VoiceNote::Speech { id, piece }
}

/// The first of two pieces of speech.
fn first_speech(id: u32) -> VoiceNote {
    spoken(id, 0, false)
}

/// The last of two pieces of speech.
fn last_speech(id: u32) -> VoiceNote {
    spoken(id, 1, true)
}

/// A piece of speech out of order.
fn stray_speech(id: u32) -> VoiceNote {
    spoken(id, 1, false)
}

/// A cloud dictation: no model, cut within the row.
fn cloud() -> SpeechAsk {
    SpeechAsk {
        piece_s: ROW_S,
        ..ask(None)
    }
}

/// A cloud run on `script`, stopped after its job told `notes`; what it heard.
fn stopped(
    speech: &WorkerSpeech<Turns>,
    script: &Script<VoiceFrames>,
    notes: Vec<fn(u32) -> VoiceNote>,
) -> (Box<dyn SpeechRun>, Receiver<SpeechNote>) {
    let (tell, heard) = ear();
    let mut run = speech.open(cloud(), tell).unwrap();
    script.act(VoiceAct::Say(notes));
    run.stop().unwrap();
    (run, heard)
}

#[test]
fn a_cloud_dictation_opens_the_voice_job_with_no_model_and_the_rows_cut() {
    let (script, _, speech) = solo();
    script.act(VoiceAct::Say(vec![opened]));
    let _run = speech.open(cloud(), ear().0).unwrap();
    let [VoiceAsk::Start { setup, .. }] = &asks(&script)[..] else {
        panic!("one start");
    };
    assert!(setup.model.is_none(), "no model");
    assert_eq!(setup.piece_s, ROW_S, "the row's cut");
    assert_eq!(script.launches(), [job(cap(&cloud()))]);
}

#[test]
fn speech_comes_back_after_done_in_wiped_pieces_and_again_for_a_retry() {
    let (script, _, speech) = solo();
    let notes: Vec<fn(u32) -> VoiceNote> = vec![shut, first_speech, last_speech, done];
    let (mut run, heard) = stopped(&speech, &script, notes);
    let told = next(&heard, 2);
    assert!(matches!(told[1], SpeechNote::Done(_)), "{told:?}");
    let pieces = run.pieces().unwrap();
    let bytes: Vec<&[u8]> = pieces.iter().map(|p| p.expose().as_slice()).collect();
    assert_eq!(bytes, [&SOUND[..], &SOUND[..]]);
    assert_eq!(run.pieces().unwrap().len(), 2, "Try again sends them again");
}

#[test]
fn nothing_comes_back_before_stop_or_after_a_cancel_or_a_close() {
    let (script, _, speech) = solo();
    let (tell, heard) = ear();
    let mut run = speech.open(cloud(), tell).unwrap();
    assert_eq!(run.pieces().err(), Some(SpeechError::NoSound), "edge 1");
    script.act(VoiceAct::Say(vec![shut]));
    run.cancel().unwrap();
    next(&heard, 1);
    assert_eq!(run.pieces().err(), Some(SpeechError::NoSound), "Cancel");
    let notes: Vec<fn(u32) -> VoiceNote> = vec![shut, last_speech, done];
    let (mut run, heard) = stopped(&speech, &script, notes);
    next(&heard, 2);
    run.close();
    assert_eq!(run.pieces().err(), Some(SpeechError::NoSound), "closed");
}

#[test]
fn a_wipe_takes_the_speech_and_speech_out_of_order_is_the_jobs_fault() {
    let (script, _, speech) = solo();
    let notes: Vec<fn(u32) -> VoiceNote> = vec![shut, first_speech, last_speech, done];
    let (mut run, heard) = stopped(&speech, &script, notes);
    next(&heard, 2);
    let _other = speech.open(cloud(), ear().0).unwrap();
    assert_eq!(next(&heard, 1), [SpeechNote::Wiped]);
    assert_eq!(run.pieces().err(), Some(SpeechError::NoSound), "wiped");
    let notes: Vec<fn(u32) -> VoiceNote> = vec![shut, stray_speech, done];
    let (mut run, heard) = stopped(&speech, &script, notes);
    next(&heard, 2);
    assert_eq!(run.pieces().err(), Some(SpeechError::Internal));
}

#[test]
fn a_run_that_heard_no_speech_has_no_pieces() {
    let (script, _, speech) = solo();
    let notes: Vec<fn(u32) -> VoiceNote> = vec![shut, done];
    let (mut run, heard) = stopped(&speech, &script, notes);
    next(&heard, 2);
    assert_eq!(run.pieces().map(|p| p.len()), Ok(0));
}

#[test]
fn a_run_with_a_model_has_no_pieces_even_after_its_end() {
    let (script, _, speech) = solo();
    let (tell, heard) = ear();
    let mut run = speech.open(ask(Some(FILE_A)), tell).unwrap();
    script.act(finished());
    run.stop().unwrap();
    next(&heard, 3);
    assert_eq!(
        run.pieces().err(),
        Some(SpeechError::NoSound),
        "as the port says"
    );
}

/// What the job tells a run in a property case.
#[derive(Clone, Debug)]
enum Step {
    Speech(u16),
    Done,
    Wipe,
}

fn a_step() -> impl Strategy<Value = Step> {
    let index = 0..u16::try_from(MAX_SPEECH_PIECES).unwrap();
    prop_oneof![
        4 => index.prop_map(Step::Speech),
        1 => Just(Step::Done),
        1 => Just(Step::Wipe),
    ]
}

proptest! {
    /// The router gives a run its speech only after `Done`, exactly the pieces told in order since its last ask, and
    /// nothing once a piece came out of order or a wipe took the recording.
    #[test]
    fn the_router_gives_speech_only_whole_and_in_order(steps in prop::collection::vec(a_step(), 0..40)) {
        let mut router = Router::default();
        let run = router.add(Box::new(|_| ()));
        router.map(run, 1);
        let (mut good, mut count, mut done, mut held) = (true, 0_u16, false, true);
        for step in steps {
            match step {
                Step::Speech(index) => {
                    router.hear(Told::Note(spoken(1, index, false)));
                    good &= !done && held && index == count;
                    count += 1;
                }
                Step::Done => {
                    router.hear(Told::Note(done_note()));
                    done |= held;
                }
                Step::Wipe => {
                    router.hear(Told::Idle);
                    held = false;
                }
            }
            let got = router.spoken(run);
            match (held, done, good) {
                (false, ..) | (true, false, _) => prop_assert_eq!(got.err(), Some(SpeechError::NoSound)),
                (true, true, false) => prop_assert_eq!(got.err(), Some(SpeechError::Internal)),
                (true, true, true) => prop_assert_eq!(got.map(|p| p.len()), Ok(usize::from(count))),
            }
        }
    }
}

/// Run 1's end, every piece told.
fn done_note() -> VoiceNote {
    done(1)
}
