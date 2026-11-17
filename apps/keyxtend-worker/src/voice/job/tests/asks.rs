//! Each ask's answer: every watched ask hears a note, Cancel keeps the recording until Close so Again can use it, a
//! new Start wipes it, a cancelled engine tells no more text, and goodbye or the end of input wipes and exits.

use super::*;

#[test]
fn every_watched_ask_hears_at_least_one_note() {
    let mut rig = Rig::plain();
    let clip = Cow::Borrowed("clip.wav");
    let script = [
        start(1),
        VoiceAsk::Stop { id: 1 },
        VoiceAsk::Stop { id: 1 },
        VoiceAsk::Keep { id: 1 },
        VoiceAsk::Cancel { id: 1 },
        VoiceAsk::Again {
            id: 2,
            setup: setup(false),
        },
        VoiceAsk::Close { id: 1 },
        VoiceAsk::Keep { id: 1 },
        VoiceAsk::Stop { id: 9 },
        VoiceAsk::Cancel { id: 9 },
        VoiceAsk::Close { id: 9 },
        VoiceAsk::Devices { id: 3 },
        VoiceAsk::Facts {
            id: 4,
            path: clip.clone(),
        },
        VoiceAsk::Check {
            id: 5,
            model: model(),
            clip,
        },
        start(6),
        VoiceAsk::Keep { id: 6 },
        start(7),
        VoiceAsk::Cancel { id: 6 },
    ];
    for ask in script {
        let (id, shown) = (ask.id(), format!("{ask:?}"));
        let notes = rig.ask(ask);
        assert!(notes.iter().any(|n| n.id() == id), "{shown}: {notes:?}");
    }
}

#[test]
fn asks_out_of_step_are_told_so() {
    let mut rig = Rig::plain();
    let out = |id| [fault(id, VoiceFault::OutOfStep)];
    assert_eq!(rig.ask(VoiceAsk::Stop { id: 9 }), out(9));
    rig.ask(start(1));
    assert_eq!(rig.ask(start(2)), out(2), "a second run while one listens");
    assert_eq!(
        rig.ask(VoiceAsk::Keep { id: 1 }),
        out(1),
        "keep while listening"
    );
    let again = VoiceAsk::Again {
        id: 3,
        setup: setup(true),
    };
    assert_eq!(rig.ask(again), out(3), "again while listening");
}

#[test]
fn cancel_keeps_the_recording_until_close_so_again_can_write_it() {
    let mut rig = Rig::new(RATE, 1, true);
    rig.ask(start(1));
    rig.speak(&loud(SEC), CHUNK);
    assert_eq!(rig.ask(VoiceAsk::Cancel { id: 1 }), [stage(1, Stage::Shut)]);
    assert!(!rig.open());
    assert_eq!(
        sounds(&rig.ask(VoiceAsk::Keep { id: 1 })).0,
        vec![LOUD; SEC]
    );
    let notes = rig.ask(VoiceAsk::Again {
        id: 2,
        setup: setup(true),
    });
    assert_eq!(notes, [piece(2, 0), fake::done(2, 1, 1000)]);
    assert_eq!(
        *rig.log.borrow(),
        [Got::Piece(vec![LOUD; SEC]), Got::End(1000)]
    );
    assert_eq!(sounds(&rig.ask(VoiceAsk::Keep { id: 2 })).0.len(), SEC);
    assert_eq!(rig.ask(VoiceAsk::Close { id: 2 }), [stage(2, Stage::Shut)]);
    for id in [1, 2] {
        assert_eq!(
            rig.ask(VoiceAsk::Keep { id }),
            [fault(id, VoiceFault::NoSound)]
        );
    }
}

#[test]
fn again_needs_a_held_recording_and_a_model() {
    let mut rig = Rig::new(RATE, 1, true);
    let again = |id, with_model| VoiceAsk::Again {
        id,
        setup: setup(with_model),
    };
    assert_eq!(rig.ask(again(1, true)), [fault(1, VoiceFault::NoSound)]);
    rig.ask(start(2));
    rig.ask(VoiceAsk::Stop { id: 2 });
    assert_eq!(rig.ask(again(3, false)), [fault(3, VoiceFault::OutOfStep)]);
}

#[test]
fn a_new_start_wipes_the_recording_held_before() {
    let mut rig = Rig::plain();
    rig.ask(start(1));
    rig.speak(&ramp(SEC), CHUNK);
    rig.ask(VoiceAsk::Stop { id: 1 });
    assert_eq!(rig.ask(start(2)), [stage(2, Stage::Open)]);
    assert_eq!(
        rig.ask(VoiceAsk::Keep { id: 1 }),
        [fault(1, VoiceFault::NoSound)]
    );
}

#[test]
fn a_cancelled_engine_tells_no_more_text() {
    let mut rig = Rig::new(RATE, 1, true);
    rig.ask(VoiceAsk::Start {
        id: 1,
        setup: setup(true),
    });
    rig.speak(&loud(SEC), CHUNK);
    rig.only(VoiceAsk::Stop { id: 1 });
    rig.only(VoiceAsk::Cancel { id: 1 });
    rig.pump();
    assert_eq!(rig.told(), [stage(1, Stage::Shut), stage(1, Stage::Shut)]);
}

#[test]
fn cancelling_a_writing_run_is_shut_even_after_its_recording_was_replaced() {
    let mut rig = Rig::new(RATE, 1, true);
    rig.ask(VoiceAsk::Start {
        id: 1,
        setup: setup(true),
    });
    rig.speak(&loud(SEC), CHUNK);
    rig.only(VoiceAsk::Stop { id: 1 });
    rig.only(hand_in(5, 0, false, &[1]));
    rig.only(VoiceAsk::Cancel { id: 1 });
    rig.pump();
    assert_eq!(rig.told(), [stage(1, Stage::Shut), stage(1, Stage::Shut)]);
}

#[test]
fn a_start_waits_until_the_engine_has_written() {
    let mut rig = Rig::new(RATE, 1, true);
    rig.ask(VoiceAsk::Start {
        id: 1,
        setup: setup(true),
    });
    rig.speak(&loud(SEC), CHUNK);
    rig.only(VoiceAsk::Stop { id: 1 });
    rig.only(start(2));
    assert_eq!(
        rig.told(),
        [stage(1, Stage::Shut), fault(2, VoiceFault::OutOfStep)]
    );
    rig.pump();
    assert_eq!(rig.told(), [piece(1, 0), fake::done(1, 1, 1000)]);
    assert_eq!(rig.ask(start(3)), [stage(3, Stage::Open)]);
}

#[test]
fn an_engine_fault_that_ends_the_run_closes_the_source_and_a_card_fault_does_not() {
    let mut rig = Rig::new(RATE, 1, true);
    rig.ask(VoiceAsk::Start {
        id: 1,
        setup: setup(true),
    });
    rig.tx
        .send(Event::Note(fault(1, VoiceFault::CardFailed)))
        .unwrap();
    rig.pump();
    assert_eq!(
        (rig.told(), rig.open()),
        (vec![fault(1, VoiceFault::CardFailed)], true)
    );
    rig.tx
        .send(Event::Note(fault(1, VoiceFault::BadFile)))
        .unwrap();
    rig.pump();
    let ended = vec![stage(1, Stage::Shut), fault(1, VoiceFault::BadFile)];
    assert_eq!((rig.told(), rig.open()), (ended, false));
    rig.tx.send(Event::Note(piece(1, 0))).unwrap();
    rig.pump();
    assert!(rig.told().is_empty(), "a note of no live writer is dropped");
}

#[test]
fn goodbye_and_the_end_of_input_wipe_and_exit() {
    for (end, want) in [
        (Event::Ask(VoiceAsk::Bye), Ok(())),
        (Event::End(Ok(())), Ok(())),
        (Event::End(Err(IpcError::BadMagic)), Err(IpcError::BadMagic)),
    ] {
        let mut rig = Rig::plain();
        rig.ask(start(1));
        rig.speak(&ramp(SEC), CHUNK);
        let flow = rig.job.take(end, &mut rig.notes).unwrap();
        assert!(matches!(flow, Flow::Exit(ref got) if *got == want));
        assert!(!rig.open() && rig.job.held.is_none() && rig.job.run.is_none());
    }
}
