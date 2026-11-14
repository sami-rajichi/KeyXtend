//! A run's life: a death told mid-run, faults the processor survives, failures once per call, Try again, Cancel's
//! silence, the held recording and its wipe at close, a `Wiped` note or a drop.

use super::*;
use std::sync::mpsc;

#[test]
fn a_death_is_told_then_the_run_answers_gone_and_its_sound_is_lost() {
    for death in [SpeechError::Gone, SpeechError::Timeout] {
        let fake = FakeSpeech::new();
        let (mut run, heard) = opened(&fake);
        assert!(fake.tell(piece(0)));
        let fault = SpeechNote::Fault(death);
        assert!(fake.tell(fault.clone()), "{death:?}");
        assert!(!fake.tell(piece(1)), "a dead worker tells no more");
        assert_eq!(notes(&heard), [piece(0), fault]);
        assert_eq!(
            (run.stop(), run.cancel()),
            (Err(SpeechError::Gone), Err(SpeechError::Gone))
        );
        assert_eq!(run.keep().err(), Some(SpeechError::NoSound));
        assert_eq!(run.again(ask_with(OTHER)), Err(SpeechError::NoSound));
        let life = &fake.runs()[0];
        assert!(life.dead && !life.held && !life.closed);
        assert!(life.agains.is_empty() && life.stops == 0);
    }
}

#[test]
fn die_is_a_told_gone_and_a_cue_can_kill_at_stop() {
    let fake = FakeSpeech::new();
    assert!(!fake.die(), "no run to kill");
    fake.on(
        SpeechCall::Stop,
        vec![piece(0), SpeechNote::Fault(SpeechError::Gone), done(1)],
    );
    let (mut run, heard) = opened(&fake);
    assert_eq!(run.stop(), Ok(()));
    let told = [piece(0), SpeechNote::Fault(SpeechError::Gone)];
    assert_eq!(notes(&heard), told, "nothing after the death");
    assert!(fake.runs()[0].dead);
    let (_next, next_heard) = opened(&fake);
    assert!(fake.die());
    assert_eq!(notes(&next_heard), [SpeechNote::Fault(SpeechError::Gone)]);
}

#[test]
fn a_fault_the_processor_survives_keeps_the_run_alive() {
    let fake = FakeSpeech::new();
    let (mut run, heard) = opened(&fake);
    let card = SpeechNote::Fault(SpeechError::CardFailed);
    assert!(fake.tell(card.clone()));
    assert!(fake.tell(piece(0)));
    assert_eq!(notes(&heard), [card, piece(0)]);
    assert_eq!(run.stop(), Ok(()));
    assert!(!fake.runs()[0].dead);
}

#[test]
fn a_failure_set_for_a_call_fails_it_once_and_tells_nothing() {
    let fake = FakeSpeech::new();
    let down = SpeechError::Down {
        retry_in: Duration::from_secs(2),
    };
    fake.fail(SpeechCall::Open, down);
    fake.on(SpeechCall::Open, vec![SpeechNote::Heard]);
    assert_eq!(fake.open(ask(), Box::new(|_| {})).err(), Some(down));
    assert!(fake.runs().is_empty());
    let (mut run, heard) = opened(&fake);
    assert_eq!(notes(&heard), [SpeechNote::Heard], "the cue waited");
    fake.fail(SpeechCall::Again, down);
    fake.fail(SpeechCall::Keep, SpeechError::NoSound);
    fake.on(SpeechCall::Again, vec![done(0)]);
    assert_eq!(run.again(ask_with(OTHER)), Err(down));
    assert_eq!(run.keep().err(), Some(SpeechError::NoSound));
    assert_eq!(notes(&heard).len(), 1, "a failed call tells nothing");
    assert_eq!(run.again(ask_with(OTHER)), Ok(()));
    assert_eq!(run.keep().map(|k| k.len()), Ok(0));
    let life = &fake.runs()[0];
    assert_eq!(
        (life.agains.len(), life.keeps),
        (1, 1),
        "failures are not counted"
    );
    assert!(!life.dead);
}

#[test]
fn a_failed_gone_or_timeout_kills_the_worker_as_the_host_does() {
    for death in [SpeechError::Gone, SpeechError::Timeout] {
        let fake = FakeSpeech::new();
        let (mut run, heard) = opened(&fake);
        fake.fail(SpeechCall::Stop, death);
        assert_eq!(run.stop(), Err(death));
        assert_eq!(run.stop(), Err(SpeechError::Gone));
        assert_eq!(run.keep().err(), Some(SpeechError::NoSound));
        assert!(!fake.tell(piece(0)), "a dead worker tells no more");
        assert!(notes(&heard).is_empty());
        let life = &fake.runs()[0];
        assert!(life.dead && !life.held && life.stops == 0);
    }
}

#[test]
fn try_again_checks_its_ask_records_the_new_model_and_tells_its_cue() {
    let fake = FakeSpeech::new();
    let (mut run, heard) = opened(&fake);
    let bare = SpeechAsk {
        model: None,
        ..ask()
    };
    assert_eq!(run.again(bare), Err(SpeechError::Range(NO_MODEL)));
    let odd = SpeechAsk {
        pad_ms: u16::MAX,
        ..ask_with(OTHER)
    };
    assert_eq!(run.again(odd), Err(SpeechError::Range("pad_ms")));
    fake.on(SpeechCall::Again, vec![piece(0), done(1)]);
    assert_eq!(run.again(ask_with(OTHER)), Ok(()));
    assert_eq!(notes(&heard), [piece(0), done(1)]);
    let life = &fake.runs()[0];
    assert_eq!(life.agains, [ask_with(OTHER)]);
    assert_ne!(life.agains[0].model, life.ask.model, "a new model");
}

#[test]
fn keep_hands_the_set_sound_and_cancel_keeps_it_for_transcribe_anyway() {
    let fake = FakeSpeech::new();
    let pieces = vec![SOUND.to_vec(), SOUND[..1].to_vec()];
    fake.set_sound(pieces.clone());
    let (mut run, _heard) = opened(&fake);
    assert_eq!(run.cancel(), Ok(()));
    let kept = run.keep().unwrap();
    let kept: Vec<Vec<u8>> = kept.into_iter().map(|k| k.expose().to_vec()).collect();
    assert_eq!(kept, pieces);
    assert_eq!(run.again(ask()), Ok(()), "the recording stays after Cancel");
    let life = &fake.runs()[0];
    assert_eq!((life.cancels, life.keeps, life.held), (1, 1, true));
}

#[test]
fn cancel_tells_no_more_text_until_try_again() {
    let fake = FakeSpeech::new();
    let shut = SpeechNote::Stage(SpeechStage::Shut);
    fake.on(SpeechCall::Cancel, vec![shut.clone(), piece(0)]);
    let (mut run, heard) = opened(&fake);
    assert_eq!(run.cancel(), Ok(()));
    assert!(!fake.tell(piece(1)), "no text after Cancel");
    assert!(fake.tell(SpeechNote::Level(0)));
    assert_eq!(run.again(ask_with(OTHER)), Ok(()));
    assert!(fake.tell(piece(0)), "Try again writes again");
    assert_eq!(notes(&heard), [shut, SpeechNote::Level(0), piece(0)]);
}

#[test]
fn close_wipes_the_sound_and_ends_the_notes() {
    let fake = FakeSpeech::new();
    fake.on(SpeechCall::Stop, vec![SpeechNote::Heard]);
    let (mut run, heard) = opened(&fake);
    run.close();
    assert_eq!(run.stop(), Ok(()), "a closed run answers and hears nothing");
    assert_eq!(run.keep().err(), Some(SpeechError::NoSound));
    assert_eq!(run.again(ask()), Err(SpeechError::NoSound));
    assert!(!fake.tell(SpeechNote::Heard));
    assert!(notes(&heard).is_empty());
    let life = &fake.runs()[0];
    assert!(life.closed && !life.held && !life.dead);
    let (mut next, next_heard) = opened(&fake);
    assert_eq!(next.stop(), Ok(()));
    assert_eq!(notes(&next_heard), [SpeechNote::Heard], "the cue waited");
}

#[test]
fn close_waits_out_a_note_being_told_and_none_comes_after() {
    let (inside, entered) = mpsc::channel();
    let (go, held) = mpsc::channel::<()>();
    let order = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&order);
    let tell = move |_: SpeechNote| {
        inside.send(()).unwrap();
        held.recv().unwrap();
        seen.lock().unwrap().push("heard");
    };
    let fake = FakeSpeech::new();
    let mut run = fake.open(ask(), Box::new(tell)).ok().unwrap();
    let teller = fake.clone();
    let teller = thread::spawn(move || teller.tell(SpeechNote::Heard));
    entered.recv_timeout(WITHIN).unwrap();
    let after = Arc::clone(&order);
    let closer = thread::spawn(move || {
        run.close();
        after.lock().unwrap().push("closed");
    });
    assert!(fake.wait(WITHIN, |runs| runs[0].closed));
    go.send(()).unwrap();
    assert!(teller.join().unwrap());
    closer.join().unwrap();
    assert_eq!(*order.lock().unwrap(), ["heard", "closed"]);
    assert!(!fake.tell(SpeechNote::Quiet));
}

#[test]
fn a_wiped_note_drops_the_sound_but_the_run_still_answers() {
    let fake = FakeSpeech::new();
    let (mut run, heard) = opened(&fake);
    assert!(fake.tell(SpeechNote::Wiped));
    assert_eq!(notes(&heard), [SpeechNote::Wiped]);
    assert_eq!(run.again(ask_with(OTHER)), Err(SpeechError::NoSound));
    assert_eq!(run.keep().err(), Some(SpeechError::NoSound));
    assert_eq!(run.stop(), Ok(()));
    assert!(!fake.runs()[0].held);
}

#[test]
fn dropping_a_run_closes_it() {
    let fake = FakeSpeech::new();
    let (run, _heard) = opened(&fake);
    drop(run);
    assert!(fake.runs()[0].closed);
    assert!(!fake.tell(SpeechNote::Heard));
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "never returns")]
fn a_failure_the_port_never_returns_is_a_test_mistake() {
    FakeSpeech::new().fail(SpeechCall::Open, SpeechError::Gone);
}
