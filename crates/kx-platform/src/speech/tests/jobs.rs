//! One job per model, capped by its row, `job_mb` and the recording's room; `idle_min` ending it; and a model change
//! moving the recording to a new job in wiped pieces under the frame cap, parked when that job cannot start.

use super::*;
use crate::speech::job::{extra_mb, room_mb};
use crate::worker::fake::voice::open as opened;
use kx_ipc::MAX_SOUND_PIECES;

#[test]
fn a_job_per_model_with_its_cap_and_job_mb_alone_without_a_model() {
    let (script, _, speech) = solo();
    let listen = ask(None);
    let record = usize::from(listen.record_max_s) * kx_ipc::SOUND_HZ * kx_ipc::SAMPLE_BYTES;
    let room = room_mb(record);
    assert_eq!(
        cap(&listen),
        limits().job_mb + room,
        "job_mb and the room alone"
    );
    drop(speech.open(listen, ear().0).unwrap());
    let with_a = ask(Some(FILE_A));
    assert_eq!(cap(&with_a), limits().job_mb + MODEL_MB + room);
    drop(speech.open(with_a.clone(), ear().0).unwrap());
    drop(speech.open(with_a, ear().0).unwrap());
    drop(speech.open(ask(Some(FILE_B)), ear().0).unwrap());
    let caps: Vec<u32> = script.launches().iter().map(|l| l.memory_mb).collect();
    let (alone, model) = (limits().job_mb + room, limits().job_mb + MODEL_MB + room);
    assert_eq!(caps, [alone, model, model], "the same model kept its job");
    assert_eq!(script.launches()[0], job(alone));
    let byes = script.calls().iter().filter(|c| **c == Call::Bye).count();
    assert_eq!(byes, 2, "each model change ended the job before");
}

#[test]
fn idle_min_ends_the_job_and_its_open_run_hears_wiped() {
    let (script, clock, speech) = solo();
    let (tell, heard) = ear();
    script.act(VoiceAct::Say(vec![opened]));
    let mut run = speech.open(ask(Some(FILE_A)), tell).unwrap();
    script.act(finished());
    run.stop().unwrap();
    assert_eq!(
        next(&heard, 4)[3],
        SpeechNote::Done(kx_module_api::ports::SpeechDone {
            pieces: 1,
            sound_ms: 0,
            wait_ms: 0,
            load_ms: 0,
            memory_mb: 0,
            card: false,
        })
    );
    clock.advance(Duration::from_secs(u64::from(limits().idle_min) * 60));
    speech.poke();
    assert_eq!(next(&heard, 1), [SpeechNote::Wiped]);
    assert!(script.calls().ends_with(&[
        Call::Bye,
        Call::Code {
            within: Duration::from_millis(limits().ask_ms.into())
        },
        Call::End
    ]));
    assert_eq!(
        run.again(ask(Some(FILE_A))),
        Err(SpeechError::NoSound),
        "Try again dims"
    );
    assert_eq!(run.keep().err(), Some(SpeechError::NoSound));
}

#[test]
fn a_model_change_hands_the_sound_over_in_wiped_pieces() {
    let (old, new, clock) = (Script::default(), Script::default(), Hands::default());
    let speech = host(&[old.clone(), new.clone()], &clock);
    let (tell, heard) = ear();
    let mut run = speech.open(ask(Some(FILE_A)), tell).unwrap();
    old.act(finished());
    run.stop().unwrap();
    next(&heard, 3);
    old.act(VoiceAct::Say(vec![first_sound, last_sound]));
    run.again(ask(Some(FILE_B))).unwrap();
    assert!(matches!(asks(&old).last(), Some(VoiceAsk::Keep { .. })));
    assert!(old.calls().contains(&Call::Bye), "the old job ended");
    assert_eq!(new.launches(), [job(cap(&ask(Some(FILE_B))))]);
    let taken = asks(&new);
    let [
        VoiceAsk::Sound { id, piece: a },
        VoiceAsk::Sound { piece: b, .. },
        again,
    ] = &taken[..]
    else {
        panic!("two pieces, then Again: {}", taken.len());
    };
    assert_eq!((a.index, a.last, &*a.bytes), (0, false, &SOUND[..]));
    assert_eq!((b.index, b.last, &*b.bytes), (1, true, &SOUND[..]));
    assert!(
        matches!((&a.bytes, &b.bytes), (Wiped::Kept(_), Wiped::Kept(_))),
        "wiped buffers"
    );
    assert!(
        matches!(again, VoiceAsk::Again { id: again_id, setup } if again_id == id
        && setup.model.as_ref().is_some_and(|m| m.path == native(FILE_B)))
    );
}

#[test]
fn again_on_the_same_model_stays_in_its_job_as_a_fresh_run() {
    let (script, _, speech) = solo();
    let (tell, heard) = ear();
    let mut run = speech.open(ask(Some(FILE_A)), tell).unwrap();
    script.act(finished());
    run.stop().unwrap();
    next(&heard, 3);
    script.act(VoiceAct::Say(vec![piece, done]));
    run.again(ask(Some(FILE_A))).unwrap();
    assert_eq!(next(&heard, 2).len(), 2, "its notes come to the same run");
    assert_eq!(script.spawns(), 1);
    let ids: Vec<u32> = asks(&script).iter().map(VoiceAsk::id).collect();
    assert_eq!(ids, [1, 1, 2], "Start, Stop, then Again on a fresh id");
}

#[test]
fn a_ten_minute_recording_crosses_in_pieces_under_the_frame_cap() {
    let (old, new, clock) = (Script::default(), Script::default(), Hands::default());
    let speech = host(&[old.clone(), new.clone()], &clock);
    let mut run = speech.open(ask(Some(FILE_A)), ear().0).unwrap();
    let full = TEN_MIN_BYTES / MAX_SOUND_BYTES;
    let mut said: Vec<fn(u32) -> VoiceNote> = vec![big_piece; full];
    said.push(big_last);
    old.act(VoiceAct::Say(said));
    run.again(ask(Some(FILE_B))).unwrap();
    let pieces: Vec<SoundPiece<'static>> = asks(&new)
        .into_iter()
        .filter_map(|a| match a {
            VoiceAsk::Sound { piece, .. } => Some(piece),
            _ => None,
        })
        .collect();
    assert_eq!(pieces.len(), full + 1);
    assert!(pieces.len() < MAX_SOUND_PIECES);
    for (at, p) in pieces.iter().enumerate() {
        assert_eq!(usize::from(p.index), at, "indexed in order");
        assert_eq!(p.last, at == full, "the last marked alone");
        assert!(p.bytes.len() <= MAX_SOUND_BYTES);
    }
    const {
        assert!(
            TEN_MIN_BYTES <= kx_ipc::MAX_RECORD_BYTES,
            "within the frames' limit"
        );
    }
    assert_eq!(
        extra_mb(TEN_MIN_BYTES, TEN_MIN_S),
        0,
        "ten minutes fit the ask's own room"
    );
}

#[test]
fn a_recording_the_new_job_cannot_take_is_parked_for_the_next_again() {
    let (old, new, clock) = (Script::default(), Script::default(), Hands::default());
    let speech = host(&[old.clone(), new.clone()], &clock);
    let mut run = speech.open(ask(Some(FILE_A)), ear().0).unwrap();
    old.act(VoiceAct::Say(vec![first_sound, last_sound]));
    new.refuse(1);
    let down = run.again(ask(Some(FILE_B)));
    assert!(matches!(down, Err(SpeechError::Down { .. })), "{down:?}");
    let kept = run.keep().unwrap();
    assert_eq!(kept.len(), 2, "still here, wiped at Close");
    clock.advance(Duration::from_millis(limits().restart_ms.into()));
    run.again(ask(Some(FILE_B))).unwrap();
    assert_eq!(asks(&new).len(), 3, "two pieces, then Again");
    assert_eq!(
        asks(&old)
            .iter()
            .filter(|a| matches!(a, VoiceAsk::Keep { .. }))
            .count(),
        1
    );
}
