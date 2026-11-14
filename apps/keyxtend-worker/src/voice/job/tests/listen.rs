//! A listening run: levels at `level_hz` from any format, the recording handed over at Stop in pieces under the cap,
//! the limit and the silence stop, a lost or refused source, and pieces reaching the engine trimmed and in order.

use super::*;
use kx_ipc::{MAX_SOUND_BYTES, RECORD_MAX_S};

#[test]
fn start_opens_the_source_and_tells_open() {
    let mut rig = Rig::plain();
    assert_eq!(rig.ask(start(1)), [stage(1, Stage::Open)]);
    assert!(rig.open());
}

#[test]
fn levels_come_at_level_hz_from_any_format() {
    for (rate, channels) in [(RATE, 1), (48_000, 2), (44_100, 1)] {
        let mut rig = Rig::new(rate, channels, false);
        rig.ask(start(1));
        let samples = 2 * usize::try_from(rate).unwrap() * usize::from(channels);
        let told = levels(&rig.speak(&vec![0.5; samples], 480));
        assert_eq!(
            told.len(),
            2 * usize::from(HZ),
            "{rate} Hz, {channels} channels"
        );
        assert!(told.iter().all(|&l| l == 128), "{told:?}");
    }
}

#[test]
fn listening_only_hands_the_recording_over_at_stop_in_pieces_under_the_cap() {
    let (mut rig, n) = (Rig::plain(), 40 * SEC);
    rig.ask(start(7));
    rig.speak(&ramp(n), 1600);
    let notes = rig.ask(VoiceAsk::Stop { id: 7 });
    assert_eq!(notes.first(), Some(&stage(7, Stage::Shut)));
    let (samples, pieces) = sounds(&notes);
    let rest = n * SAMPLE_BYTES - MAX_SOUND_BYTES;
    assert_eq!(pieces, [(0, false, MAX_SOUND_BYTES), (1, true, rest)]);
    assert_eq!(samples, ramp16(n));
    assert_eq!(notes.last(), Some(&fake::done(7, 0, 40_000)));
    assert_eq!(notes.len(), 4);
    assert!(!rig.open());
}

#[test]
fn the_limit_stops_the_run_at_exactly_record_max_s() {
    let mut rig = Rig::plain();
    let mut limited = setup(false);
    limited.record_max_s = *RECORD_MAX_S.start();
    rig.ask(VoiceAsk::Start {
        id: 1,
        setup: limited,
    });
    let max = usize::from(*RECORD_MAX_S.start()) * SEC;
    let notes = plain(&rig.speak(&ramp(max + 2 * SEC), CHUNK));
    assert_eq!(notes.first(), Some(&stage(1, Stage::Limit)));
    assert_eq!(sounds(&notes).0, ramp16(max));
    assert_eq!(notes.last(), Some(&fake::done(1, 0, 10_000)));
    assert!(!rig.open());
    assert!(
        rig.speak(&ramp(SEC), CHUNK).is_empty(),
        "late chunks are dropped"
    );
    assert_eq!(rig.ask(VoiceAsk::Stop { id: 1 }), [stage(1, Stage::Shut)]);
}

#[test]
fn a_stalled_job_queues_no_more_sound_than_its_run_can_record() {
    let max_s = *RECORD_MAX_S.start();
    for (rate, channels) in [(8000, 1), (RATE, 1), (48_000, 2)] {
        let mut rig = Rig::new(rate, channels, false);
        let mut limited = setup(false);
        limited.record_max_s = max_s;
        rig.ask(VoiceAsk::Start {
            id: 1,
            setup: limited,
        });
        let per_s = usize::try_from(rate).unwrap() * usize::from(channels);
        let (feed, chunk) = (rig.mic.borrow().feed.clone().unwrap(), per_s / 100);
        for _ in 0..(usize::from(max_s) + 5) * 100 {
            feed.sound(Buf::collect(vec![0.25; chunk].into_iter()).unwrap());
        }
        let queued: Vec<Event> = rig.rx.try_iter().collect();
        let size = |e: &Event| match e {
            Event::Sound { chunk, .. } => chunk.len(),
            _ => 0,
        };
        let samples: usize = queued.iter().map(size).sum();
        let most = (usize::from(max_s) + 1) * per_s + chunk;
        assert!(samples <= most, "{rate} Hz: {samples} queued");
        for event in queued {
            rig.job.take(event, &mut rig.notes).unwrap();
        }
        let notes = plain(&rig.told());
        assert_eq!(notes.first(), Some(&stage(1, Stage::Limit)), "{rate} Hz");
        assert_eq!(sounds(&notes).0.len(), usize::from(max_s) * SEC);
    }
}

#[test]
fn the_silence_stop_waits_for_speech_then_stops_after_its_silence() {
    let mut rig = Rig::new(RATE, 1, true);
    let mut quiet = setup(false);
    quiet.silence_stop_s = Some(1);
    rig.ask(VoiceAsk::Start {
        id: 1,
        setup: quiet,
    });
    assert!(
        plain(&rig.speak(&silent(2 * SEC), CHUNK)).is_empty(),
        "no speech yet"
    );
    let heard = rig.speak(&loud(SEC), CHUNK);
    assert!(heard.contains(&VoiceNote::Heard {
        id: 1,
        speech: true
    }));
    let notes = rig.speak(&silent(2 * SEC), CHUNK);
    assert!(notes.contains(&VoiceNote::Heard {
        id: 1,
        speech: false
    }));
    let notes = plain(&notes);
    assert_eq!(notes.first(), Some(&stage(1, Stage::Silence)));
    assert_eq!(
        sounds(&notes).0.len(),
        4 * SEC,
        "two silent, one loud, one silent"
    );
}

#[test]
fn with_the_silence_stop_off_a_run_listens_on() {
    let mut rig = Rig::new(RATE, 1, true);
    rig.ask(start(1));
    rig.speak(&loud(SEC), CHUNK);
    assert!(plain(&rig.speak(&silent(5 * SEC), CHUNK)).is_empty());
    assert!(rig.open());
}

#[test]
fn a_lost_source_tells_shut_then_its_fault_and_keeps_the_recording() {
    let mut rig = Rig::plain();
    rig.ask(start(1));
    rig.speak(&ramp(SEC), CHUNK);
    rig.mic
        .borrow()
        .feed
        .clone()
        .unwrap()
        .lost(VoiceFault::Unplugged);
    rig.pump();
    let shut = [stage(1, Stage::Shut), fault(1, VoiceFault::Unplugged)];
    assert_eq!(rig.told(), shut);
    assert!(!rig.open());
    assert_eq!(sounds(&rig.ask(VoiceAsk::Keep { id: 1 })).0, ramp16(SEC));
}

#[test]
fn a_source_that_will_not_open_is_its_fault_and_leaves_no_run() {
    let mut rig = Rig::plain();
    rig.mic.borrow_mut().refuse = Some(VoiceFault::MicBusy);
    assert_eq!(rig.ask(start(1)), [fault(1, VoiceFault::MicBusy)]);
    rig.mic.borrow_mut().refuse = None;
    assert_eq!(rig.ask(start(2)), [stage(2, Stage::Open)]);
}

#[test]
fn pieces_reach_the_engine_trimmed_and_in_order_while_listening_goes_on() {
    let mut rig = Rig::new(RATE, 1, true);
    rig.ask(VoiceAsk::Start {
        id: 1,
        setup: setup(true),
    });
    let mut talk = loud(12 * SEC);
    talk.extend(silent(SEC));
    talk.extend(loud(3 * SEC));
    let notes = plain(&rig.speak(&talk, CHUNK));
    assert_eq!(
        notes,
        [piece(1, 0)],
        "the first piece is written while listening goes on"
    );
    let notes = rig.ask(VoiceAsk::Stop { id: 1 });
    assert_eq!(
        notes,
        [stage(1, Stage::Shut), piece(1, 1), fake::done(1, 2, 16_000)]
    );
    // The cut falls 300 ms into the pause; the second piece is trimmed to 300 ms before its speech.
    let pad = SEC * 3 / 10;
    let want = [
        Got::Piece(
            vec![LOUD; 12 * SEC]
                .into_iter()
                .chain(vec![0; pad])
                .collect(),
        ),
        Got::Piece(
            vec![0; pad]
                .into_iter()
                .chain(vec![LOUD; 3 * SEC])
                .collect(),
        ),
        Got::End(16_000),
    ];
    assert_eq!(*rig.log.borrow(), want);
}

#[test]
fn a_model_without_a_detector_is_runner_and_opens_nothing() {
    let mut rig = Rig::plain();
    let ask = VoiceAsk::Start {
        id: 1,
        setup: setup(true),
    };
    assert_eq!(rig.ask(ask), [fault(1, VoiceFault::Runner)]);
    assert_eq!(rig.mic.borrow().opens, 0);
}
