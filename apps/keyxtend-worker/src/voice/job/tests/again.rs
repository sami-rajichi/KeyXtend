//! Again on a long held recording: it is cut a stretch per turn of the job's loop, so an ask taken between two
//! stretches is answered at once, a Cancel stops the cutting, and a hand-in waits until the cutting is done.

use super::*;
use kx_ipc::MAX_SOUND_BYTES;

/// Six times 11 s of speech and 1 s of silence: six pieces of 10 s or more.
fn talk() -> Vec<i16> {
    let mut out = Vec::new();
    for _ in 0..6 {
        out.extend(vec![LOUD; 11 * SEC]);
        out.extend(vec![0; SEC]);
    }
    out
}

/// A job holding `talk()` handed in as run 5.
fn holding() -> Rig {
    let mut rig = Rig::new(RATE, 1, true);
    let sound = talk();
    let parts: Vec<&[i16]> = sound.chunks(MAX_SOUND_BYTES / SAMPLE_BYTES).collect();
    for (k, part) in parts.iter().enumerate() {
        let index = u16::try_from(k).unwrap();
        assert!(
            rig.ask(hand_in(5, index, k + 1 == parts.len(), part))
                .is_empty()
        );
    }
    rig
}

fn again(id: u32) -> VoiceAsk<'static> {
    VoiceAsk::Again {
        id,
        setup: setup(true),
    }
}

#[test]
fn again_gives_every_piece_of_a_long_recording_then_ends() {
    let mut rig = holding();
    let notes = rig.ask(again(6));
    let ms = span::heard_ms(talk().len());
    let mut want: Vec<VoiceNote> = (0..6).map(|k| piece(6, k)).collect();
    want.push(fake::done(6, 6, ms));
    assert_eq!(notes, want);
    assert_eq!(rig.log.borrow().last(), Some(&Got::End(ms)));
}

#[test]
fn a_cancel_between_two_stretches_is_answered_at_once_and_stops_the_cutting() {
    let mut rig = holding();
    rig.only(again(6));
    rig.tx.send(Event::Ask(VoiceAsk::Cancel { id: 6 })).unwrap();
    rig.pump();
    assert_eq!(rig.told(), [stage(6, Stage::Shut)]);
    assert!(
        rig.log.borrow().is_empty(),
        "nothing was given past the cancel"
    );
}

#[test]
fn a_hand_in_while_again_still_cuts_is_out_of_step() {
    let mut rig = holding();
    rig.only(again(6));
    rig.only(hand_in(7, 0, true, &[1]));
    assert_eq!(rig.told(), [fault(7, VoiceFault::OutOfStep)]);
    rig.pump();
    assert_eq!(rig.told().len(), 7, "six pieces and the end");
    assert!(rig.ask(hand_in(7, 0, true, &[1])).is_empty());
}
