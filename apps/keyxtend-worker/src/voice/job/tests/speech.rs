//! A listen-only run's speech at Stop: cut at every pause, trimmed to its pad, packed in order within `piece_s`, while
//! the held recording stays whole; nothing goes before Stop or after a Cancel (edge 1), and ten minutes go in several
//! pieces (edge 8).

use super::*;
use kx_ipc::RECORD_MAX_S;

/// Starts run `id` listening only, packing within `piece_s`, with a limit past what the test speaks.
fn listen(rig: &mut Rig, id: u32, piece_s: u8) {
    let setup = Setup {
        piece_s,
        record_max_s: *RECORD_MAX_S.end(),
        ..setup(false)
    };
    rig.ask(VoiceAsk::Start { id, setup });
}

/// Speaks each `(loud, seconds)` in order; the notes told.
fn say(rig: &mut Rig, parts: &[(bool, usize)]) -> Vec<VoiceNote> {
    let mut told = Vec::new();
    for &(spoken, secs) in parts {
        let sound = if spoken {
            loud(secs * SEC)
        } else {
            silent(secs * SEC)
        };
        told.extend(rig.speak(&sound, CHUNK));
    }
    told
}

#[test]
fn speech_is_trimmed_to_its_pad_and_packed_within_piece_s_while_the_recording_stays_whole() {
    let mut rig = Rig::new(RATE, 1, true);
    listen(&mut rig, 1, *PIECE_S.start());
    let talk = [
        (false, 2),
        (true, 3),
        (false, 1),
        (true, 4),
        (false, 2),
        (true, 6),
        (false, 1),
    ];
    say(&mut rig, &talk);
    let notes = rig.ask(VoiceAsk::Stop { id: 1 });
    let (samples, pieces) = speech(&notes);
    let bytes = |secs: usize| (secs * SEC + 2 * PAD) * SAMPLE_BYTES;
    let joined = bytes(3) + bytes(4);
    assert_eq!(pieces, [(0, false, joined), (1, true, bytes(6))]);
    let spoken = samples.iter().filter(|&&s| s == LOUD).count();
    assert_eq!(spoken, 13 * SEC, "every spoken sample, in order");
    assert_eq!(notes.last(), Some(&fake::done(1, 0, 19_000)));
    let kept = sounds(&rig.ask(VoiceAsk::Keep { id: 1 })).0;
    assert_eq!(kept.len(), 19 * SEC, "the recording stays whole");
}

#[test]
fn nothing_is_handed_over_while_listening_or_after_a_cancel() {
    let mut rig = Rig::new(RATE, 1, true);
    listen(&mut rig, 1, *PIECE_S.start());
    let heard = say(&mut rig, &[(true, 12), (false, 1)]);
    assert!(speech(&heard).1.is_empty(), "nothing before Stop (edge 1)");
    let notes = rig.ask(VoiceAsk::Cancel { id: 1 });
    assert!(speech(&notes).1.is_empty(), "a Cancel hands over nothing");
    let kept = sounds(&rig.ask(VoiceAsk::Keep { id: 1 })).0;
    assert_eq!(kept.len(), 13 * SEC, "the recording is held");
}

#[test]
fn a_ten_minute_dictation_goes_in_several_pieces_each_within_piece_s() {
    let (mut rig, piece_s) = (Rig::new(RATE, 1, true), 30);
    listen(&mut rig, 1, piece_s);
    let minute = [(true, 4), (false, 1)].repeat(12);
    let mut notes = Vec::new();
    for _ in 0..10 {
        notes.extend(say(&mut rig, &minute));
    }
    notes.extend(rig.ask(VoiceAsk::Stop { id: 1 }));
    let (_, sent) = speech(&notes);
    assert_eq!(sent.len(), 20, "six sentences a piece (edge 8)");
    let most = usize::from(piece_s) * SEC * SAMPLE_BYTES;
    assert!(
        sent.iter().all(|&(_, _, n)| n <= most),
        "each within piece_s"
    );
    let order: Vec<u16> = sent.iter().map(|p| p.0).collect();
    assert_eq!(order, (0..20).collect::<Vec<u16>>());
    let lasts: Vec<bool> = sent.iter().map(|p| p.1).collect();
    assert_eq!(lasts, [vec![false; 19], vec![true]].concat());
}
