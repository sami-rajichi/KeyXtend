//! A run's life on its job: notes told as the port's, the job's peak as a run's memory, threads resolved to the real
//! cores, a worker death, silence or a broken keep as a fault that never hangs (edge 13), and a close after which
//! nothing is told.

use super::*;
use crate::worker::fake::voice::{level, open as opened};
use kx_module_api::ports::{CardId, CardUse, SpeechDone};

/// Speech heard, then silence, for run `id`.
fn heard_then_quiet() -> VoiceAct {
    VoiceAct::Say(vec![
        |id| VoiceNote::Heard { id, speech: true },
        |id| VoiceNote::Heard { id, speech: false },
        |id| VoiceNote::Warm { id, percent: 40 },
    ])
}

#[test]
fn a_runs_notes_are_told_as_the_ports_with_the_jobs_peak_as_its_memory() {
    let (script, _, speech) = solo();
    script.peak(Some(MODEL_MB));
    let (tell, heard) = ear();
    script.act(heard_then_quiet());
    let mut run = speech.open(ask(Some(FILE_A)), tell).unwrap();
    let warming = SpeechNote::Warming { percent: 40 };
    assert_eq!(
        next(&heard, 3),
        [SpeechNote::Heard, SpeechNote::Quiet, warming]
    );
    script.act(finished());
    run.stop().unwrap();
    let told = next(&heard, 3);
    assert_eq!(told[0], SpeechNote::Stage(Stage::Shut));
    assert!(matches!(&told[1], SpeechNote::Piece { index: 0, text } if !text.expose().is_empty()));
    let ended = SpeechDone {
        pieces: 1,
        sound_ms: 0,
        wait_ms: 0,
        load_ms: 0,
        memory_mb: MODEL_MB,
        card: false,
    };
    assert_eq!(told[2], SpeechNote::Done(ended), "read from the job's peak");
}

#[test]
fn threads_zero_are_the_real_cores_and_more_are_capped_before_the_ask() {
    let (script, _, speech) = solo();
    for (asked, sent) in [(0, CORES), (2, 2), (u8::MAX, CORES)] {
        let mut dictation = ask(Some(FILE_A));
        dictation.threads = asked.min(*SpeechAsk::THREADS.end());
        drop(speech.open(dictation, ear().0).unwrap());
        let started = asks(&script).into_iter().rev().find_map(|a| match a {
            VoiceAsk::Start { setup, .. } => setup.model.map(|m| m.threads),
            _ => None,
        });
        let want = u8::try_from(sent).unwrap().min(*SpeechAsk::THREADS.end());
        assert_eq!(started, Some(want), "asked {asked}");
    }
}

#[test]
fn a_card_run_names_its_card_and_needs_its_memory_and_margin() {
    let (script, _, speech) = solo();
    let mut dictation = ask(Some(FILE_A));
    let id = CardId::new(CARD).unwrap();
    dictation.card = Some(CardUse {
        id,
        need_mb: 900,
        margin_mb: 512,
    });
    drop(speech.open(dictation, ear().0).unwrap());
    let VoiceAsk::Start { setup, .. } = &asks(&script)[0] else {
        panic!("a start");
    };
    let target = setup.model.as_ref().map(|m| m.target.clone());
    let card = kx_ipc::Target::Card {
        id: CARD.into(),
        need_mb: 1_412,
    };
    assert_eq!(target, Some(card));
    assert_eq!(setup.detector, DETECTOR, "the shipped detector");
}

#[test]
fn after_a_worker_death_the_pieces_stay_keep_and_again_say_no_sound_and_stop_and_cancel_gone() {
    let (script, _, speech) = solo();
    let (tell, heard) = ear();
    script.act(VoiceAct::Say(vec![opened, piece]));
    let mut run = speech.open(ask(Some(FILE_A)), tell).unwrap();
    assert_eq!(next(&heard, 2).len(), 2, "the piece already told");
    script.quit(3);
    assert_eq!(next(&heard, 1), [SpeechNote::Fault(SpeechError::Gone)]);
    assert_eq!(run.keep().err(), Some(SpeechError::NoSound));
    assert_eq!(run.again(ask(Some(FILE_A))), Err(SpeechError::NoSound));
    assert_eq!(run.stop(), Err(SpeechError::Gone));
    assert_eq!(run.cancel(), Err(SpeechError::Gone));
    let none = SpeechError::Range(NO_MODEL);
    assert_eq!(run.again(ask(None)), Err(none), "no model is refused first");
}

#[test]
fn a_worker_silent_past_its_wait_is_a_timeout_and_never_a_hang() {
    let (_, clock, speech) = solo();
    let (tell, heard) = ear();
    let mut run = speech.open(ask(Some(FILE_A)), tell).unwrap();
    clock.advance(Duration::from_millis(limits().ask_ms.into()));
    speech.poke();
    assert_eq!(next(&heard, 1), [SpeechNote::Fault(SpeechError::Timeout)]);
    assert_eq!(run.stop(), Err(SpeechError::Gone));
}

#[test]
fn close_asks_the_job_to_wipe_and_nothing_is_told_after_it() {
    let (script, _, speech) = solo();
    let (tell, heard) = ear();
    script.act(VoiceAct::Say(vec![opened]));
    let mut run = speech.open(ask(Some(FILE_A)), tell).unwrap();
    next(&heard, 1);
    script.act(VoiceAct::Say(vec![shut]));
    run.close();
    assert!(matches!(
        asks(&script).last(),
        Some(VoiceAsk::Close { id: 1 })
    ));
    script.say(Ok(level(1)));
    run.close();
    drop(run);
    let late = heard.recv_timeout(Duration::from_millis(1));
    assert!(late.is_err(), "nothing after close: {late:?}");
    assert_eq!(asks(&script).len(), 2, "a second close sends nothing");
}

#[test]
fn a_new_dictation_wipes_the_recording_an_open_run_held() {
    let (_, _, speech) = solo();
    let (tell, heard) = ear();
    let mut first = speech.open(ask(Some(FILE_A)), tell).unwrap();
    let _second = speech.open(ask(Some(FILE_A)), ear().0).unwrap();
    assert_eq!(next(&heard, 1), [SpeechNote::Wiped]);
    assert_eq!(first.keep().err(), Some(SpeechError::NoSound));
}

#[test]
fn a_job_that_holds_no_recording_dims_keep_and_again() {
    let (script, _, speech) = solo();
    let mut run = speech.open(ask(Some(FILE_A)), ear().0).unwrap();
    script.act(VoiceAct::Say(vec![no_sound]));
    assert_eq!(run.keep().err(), Some(SpeechError::NoSound));
    let asked = asks(&script).len();
    assert_eq!(run.again(ask(Some(FILE_A))), Err(SpeechError::NoSound));
    assert_eq!(asks(&script).len(), asked, "nothing more asked");
}

#[test]
fn a_keep_the_job_answers_with_another_fault_returns_it_and_never_hangs() {
    let (script, _, speech) = solo();
    let (tell, heard) = ear();
    let mut run = speech.open(ask(Some(FILE_A)), tell).unwrap();
    script.act(VoiceAct::Say(vec![failed]));
    assert_eq!(run.keep().err(), Some(SpeechError::Internal));
    assert_eq!(next(&heard, 1), [SpeechNote::Fault(SpeechError::Internal)]);
}

#[test]
fn a_keep_past_the_longest_recording_fails_and_keeps_no_more() {
    let (script, _, speech) = solo();
    let mut run = speech.open(ask(Some(FILE_A)), ear().0).unwrap();
    let flood: fn(u32) -> VoiceNote = first_sound;
    script.act(VoiceAct::Say(vec![flood; kx_ipc::MAX_SOUND_PIECES]));
    assert_eq!(run.keep().err(), Some(SpeechError::Internal));
}

/// The job failed `id` inside.
fn failed(id: u32) -> VoiceNote {
    let fault = VoiceFault::Internal;
    VoiceNote::Fault { id, fault }
}

#[test]
fn a_run_closed_from_inside_its_own_callback_hears_nothing_more_and_never_hangs() {
    let (script, _, speech) = solo();
    let slot: Arc<Mutex<Option<Box<dyn SpeechRun>>>> = Arc::default();
    let (said, heard) = mpsc::channel();
    let inner = Arc::clone(&slot);
    let tell: Tell = Box::new(move |note| {
        let _ = said.send(note);
        if let Some(run) = lock(&inner).as_mut() {
            run.close();
        }
    });
    *lock(&slot) = Some(speech.open(ask(Some(FILE_A)), tell).unwrap());
    script.say(Ok(level(1)));
    script.say(Ok(level(1)));
    assert_eq!(next(&heard, 1), [SpeechNote::Level(1)]);
    let late = heard.recv_timeout(Duration::from_millis(1));
    assert!(late.is_err(), "nothing after its close: {late:?}");
    assert!(
        matches!(asks(&script).last(), Some(VoiceAsk::Start { .. })),
        "no ask from its thread"
    );
}
