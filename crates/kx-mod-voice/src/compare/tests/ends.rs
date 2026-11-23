//! How a Compare run ends: the recording lost, nothing heard, Cancel, the job stopping by itself, a card failing
//! mid-run, and no ready model.

use super::*;

#[test]
fn a_lost_recording_ends_with_the_models_already_written() {
    let (mut compare, take) = recorded(vec![entrant(1, false), entrant(2, false)], SET);
    compare.note(take, done(0, 0));
    compare.note(take, piece(SET));
    compare.note(take, done(900, 800));
    let end = compare.note(take, SpeechNote::Wiped);
    assert!(
        end.iter()
            .any(|a| matches!(a, Act::Save(b) if b.scores.len() == 1))
    );
    assert_eq!(compare.results().len(), 1);
}

#[test]
fn nothing_heard_runs_no_model() {
    let mut compare = Compare::default();
    let acts = compare.start(plan(SET, vec![entrant(1, false)]), &settings());
    let Some(Act::Open { take, .. }) = acts.first().cloned() else {
        panic!("no open");
    };
    let stop = compare.stop();
    assert_eq!(
        stop,
        [
            Act::Run {
                take,
                call: RunCall::Cancel
            },
            Act::Run {
                take,
                call: RunCall::Close
            }
        ]
    );
    assert_eq!(compare.note_shown(), Some(BarHint::Nothing));
    assert!(!compare.busy());
}

#[test]
fn cancel_stops_and_closes_the_run() {
    let (mut compare, take) = recorded(vec![entrant(1, false)], SET);
    compare.note(take, done(0, 0));
    let acts = compare.cancel();
    assert_eq!(
        acts,
        [
            Act::Run {
                take,
                call: RunCall::Cancel
            },
            Act::Run {
                take,
                call: RunCall::Close
            }
        ]
    );
    assert!(!compare.busy());
    assert!(
        compare.note(take, done(9, 9)).is_empty(),
        "a closed run's notes are dropped"
    );
}

#[test]
fn the_job_stopping_by_itself_ends_the_recording() {
    let mut compare = Compare::default();
    let acts = compare.start(plan(SET, vec![entrant(1, false)]), &settings());
    let Some(Act::Open { take, .. }) = acts.first().cloned() else {
        panic!("no open");
    };
    assert!(compare.recording());
    assert_eq!(compare.note(take, SpeechNote::Level(40)), [Act::Level(40)]);
    compare.note(take, SpeechNote::Heard);
    compare.note(take, SpeechNote::Stage(SpeechStage::Limit));
    assert!(!compare.recording());
    assert_eq!(agains(&compare.note(take, done(0, 0))).len(), 1);
}

#[test]
fn a_card_failing_mid_run_drops_its_check_and_the_processor_goes_on() {
    let (mut compare, take) = recorded(vec![entrant(1, true)], SET);
    compare.note(take, done(0, 0));
    let fell = compare.note(take, SpeechNote::Fault(SpeechError::CardFailed));
    let card = CardId::new("Vulkan0").unwrap();
    assert_eq!(
        fell,
        [Act::Fell {
            card,
            model: print(1)
        }]
    );
    assert!(compare.busy(), "the processor writes the same recording");
}

#[test]
fn a_start_with_no_ready_model_does_nothing() {
    let mut compare = Compare::default();
    assert!(compare.start(plan(SET, Vec::new()), &settings()).is_empty());
    assert!(!compare.busy());
}

#[test]
fn the_job_stopping_by_itself_with_no_speech_runs_no_model() {
    let mut compare = Compare::default();
    let acts = compare.start(plan(SET, vec![entrant(1, false)]), &settings());
    let Some(Act::Open { take, .. }) = acts.first().cloned() else {
        panic!("no open");
    };
    let end = compare.note(take, SpeechNote::Stage(SpeechStage::Limit));
    assert_eq!(end.len(), 2, "cancel, then close: {end:?}");
    assert_eq!(compare.note_shown(), Some(BarHint::Nothing));
    assert_eq!(compare.lend(), Lend::Off);
}

#[test]
fn a_microphone_fault_while_recording_names_the_microphone() {
    let mut compare = Compare::default();
    let acts = compare.start(plan(SET, vec![entrant(1, false)]), &settings());
    let Some(Act::Open { take, .. }) = acts.first().cloned() else {
        panic!("no open");
    };
    compare.note(take, SpeechNote::Fault(SpeechError::NoMic));
    assert_eq!(compare.note_shown(), Some(BarHint::NoMic));
    assert!(!compare.busy());
}

#[test]
fn the_lend_follows_the_run_and_a_card_fallback_names_the_processor() {
    let mut compare = Compare::default();
    assert_eq!(compare.lend(), Lend::Off);
    let acts = compare.start(plan(SET, vec![entrant(1, true)]), &settings());
    assert_eq!(
        compare.lend(),
        Lend::Prep,
        "no Listening until its microphone opens"
    );
    let Some(Act::Open { take, .. }) = acts.first().cloned() else {
        panic!("no open");
    };
    compare.note(take, SpeechNote::Stage(SpeechStage::Open));
    assert_eq!(compare.lend(), Lend::Listen);
    let (mut compare, take) = recorded(vec![entrant(1, true)], SET);
    assert_eq!(compare.lend(), Lend::Write);
    compare.note(take, done(0, 0));
    compare.note(take, piece(SET));
    let last = compare.note(take, done(900, 800));
    assert_eq!(compare.lend(), Lend::Off);
    assert_eq!(
        compare.served(),
        None,
        "the card's run fell back to the processor"
    );
    let Some(Act::Save(board)) = last.iter().find(|a| matches!(a, Act::Save(_))) else {
        panic!("no save: {last:?}");
    };
    assert_eq!(board.card, None);
    let (mut on_card, take) = recorded(vec![entrant(1, true)], SET);
    on_card.note(take, done(0, 0));
    let SpeechNote::Done(mut ran) = done(900, 800) else {
        unreachable!("done gives a Done");
    };
    ran.card = true;
    on_card.note(take, SpeechNote::Done(ran));
    assert_eq!(on_card.served(), Some(&CardId::new("Vulkan0").unwrap()));
}

#[test]
fn a_done_while_recording_with_no_speech_heard_sends_nothing_and_runs_no_model() {
    let mut compare = Compare::default();
    let plan = Plan {
        clouds: vec![remote(1)],
        ..plan(SET, vec![entrant(1, false)])
    };
    let acts = compare.start(plan, &settings());
    let Some(Act::Open { take, .. }) = acts.first().cloned() else {
        panic!("no open");
    };
    let end = compare.note(take, done(0, 0));
    assert!(
        !end.iter().any(|a| matches!(a, Act::Send { .. })),
        "{end:?}"
    );
    assert_eq!(compare.note_shown(), Some(BarHint::Nothing));
    assert!(!compare.busy());
}
