//! The hand-in: pieces of a moved recording are taken in order and serve Keep and Again; a piece out of order drops
//! the hand-in; an empty recording is one empty last piece.

use super::*;
use kx_ipc::MAX_SOUND_BYTES;

/// Samples in one handed-in piece at the cap.
const FULL: usize = MAX_SOUND_BYTES / SAMPLE_BYTES;

#[test]
fn a_recording_handed_in_serves_keep_and_again() {
    let mut rig = Rig::new(RATE, 1, true);
    let sound = vec![LOUD; FULL + SEC];
    assert!(rig.ask(hand_in(5, 0, false, &sound[..FULL])).is_empty());
    assert!(rig.ask(hand_in(5, 1, true, &sound[FULL..])).is_empty());
    let (samples, pieces) = sounds(&rig.ask(VoiceAsk::Keep { id: 5 }));
    assert_eq!(samples, sound);
    assert_eq!(
        pieces,
        [(0, false, MAX_SOUND_BYTES), (1, true, SEC * SAMPLE_BYTES)]
    );
    let notes = rig.ask(VoiceAsk::Again {
        id: 6,
        setup: setup(true),
    });
    let ms = span::heard_ms(sound.len());
    assert_eq!(notes, [piece(6, 0), fake::done(6, 1, ms)]);
    assert_eq!(rig.ask(VoiceAsk::Close { id: 5 }), [stage(5, Stage::Shut)]);
    assert_eq!(
        rig.ask(VoiceAsk::Keep { id: 6 }),
        [fault(6, VoiceFault::NoSound)]
    );
}

#[test]
fn a_piece_out_of_order_is_out_of_step_and_drops_the_hand_in() {
    let mut rig = Rig::plain();
    let out = |id| vec![fault(id, VoiceFault::OutOfStep)];
    assert_eq!(rig.ask(hand_in(5, 1, true, &[1])), out(5));
    assert!(rig.ask(hand_in(5, 0, false, &[1, 2])).is_empty());
    assert_eq!(
        rig.ask(hand_in(8, 1, true, &[3])),
        out(8),
        "another run's piece"
    );
    assert_eq!(
        rig.ask(hand_in(5, 1, true, &[3])),
        out(5),
        "the hand-in was dropped"
    );
    assert_eq!(
        rig.ask(VoiceAsk::Keep { id: 5 }),
        [fault(5, VoiceFault::NoSound)]
    );
}

#[test]
fn a_hand_in_while_listening_is_out_of_step() {
    let mut rig = Rig::plain();
    rig.ask(start(1));
    assert_eq!(
        rig.ask(hand_in(2, 0, true, &[1])),
        [fault(2, VoiceFault::OutOfStep)]
    );
}

#[test]
fn an_empty_recording_is_one_empty_last_piece() {
    let mut rig = Rig::plain();
    rig.ask(start(1));
    let notes = rig.ask(VoiceAsk::Stop { id: 1 });
    let empty = SoundPiece {
        index: 0,
        last: true,
        bytes: kx_ipc::Wiped::Lent(&[]),
    };
    let want = [
        stage(1, Stage::Shut),
        VoiceNote::Sound {
            id: 1,
            piece: empty,
        },
        fake::done(1, 0, 0),
    ];
    assert_eq!(notes, want);
}
