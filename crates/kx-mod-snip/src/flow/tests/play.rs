//! Play at the speed picked: as recorded at 1×, else the re-timed copy, made once per speed while the keys wait, the
//! one that asked told, and reused by Play, Copy and Save. A speed job that fails notes and the strip stays.

use super::reel::{key, strip, video_keep};
use super::*;

/// The re-timing `outs` asks for Play: the recording and its copy.
fn retime(outs: &[Out]) -> Option<(Video, Timed)> {
    outs.iter().find_map(|o| match o {
        Out::Retime { video, timed } => Some((*video, *timed)),
        _ => None,
    })
}

/// A stopped recording on its strip, after `steps` clicks of Speed.
fn sped(steps: usize) -> (Rig, Video) {
    let mut rig = Rig::new();
    let video = rig.stopped();
    for _ in 0..steps {
        rig.on(key(StripKey::Speed));
    }
    (rig, video)
}

/// Play at the speed shown, its copy made: the copy it plays.
fn played(rig: &mut Rig) -> Video {
    let (_, timed) = retime(&rig.on(key(StripKey::Play))).unwrap();
    assert_eq!(rig.on(In::Kept(Outcome::Done)), [Out::Play(timed.into)]);
    timed.into
}

#[test]
fn play_at_a_speed_makes_its_copy_first_then_plays_it_and_the_strip_stays() {
    let (mut rig, video) = sped(1);
    let outs = rig.on(key(StripKey::Play));
    let (from, timed) = retime(&outs).unwrap();
    assert_eq!((from, timed.speed), (video, speed(2)));
    assert_ne!(timed.into, video);
    assert!(!outs.iter().any(|o| matches!(o, Out::Play(_))), "{outs:?}");
    assert_eq!(rig.on(In::Kept(Outcome::Done)), [Out::Play(timed.into)]);
    assert_eq!(rig.stage(), &strip(video, speed(2), None));
    assert!(!rig.flow.asking());
}

#[test]
fn keys_esc_and_the_snip_key_wait_while_plays_copy_is_made() {
    let (mut rig, video) = sped(2);
    rig.on(key(StripKey::Play));
    let keys = [
        StripKey::Speed,
        StripKey::Play,
        StripKey::Copy,
        StripKey::Save,
        StripKey::Close,
    ];
    for k in keys {
        assert_eq!(rig.on(key(k)), [], "{k:?}");
    }
    assert_eq!((rig.on(In::Esc), rig.on(In::Key)), (vec![], vec![]));
    assert_eq!(rig.stage(), &strip(video, speed(4), None));
}

#[test]
fn a_second_play_and_a_copy_or_save_at_that_speed_reuse_plays_copy() {
    for (k, keep) in [(StripKey::Copy, Keep::Copy), (StripKey::Save, Keep::Save)] {
        let (mut rig, _) = sped(1);
        let copy = played(&mut rig);
        assert_eq!(rig.on(key(StripKey::Play)), [Out::Play(copy)], "made once");
        let outs = rig.on(key(k));
        assert_eq!(video_keep(&outs), Some((keep, copy, None)), "{k:?}");
    }
}

#[test]
fn a_copy_or_save_at_a_speed_makes_the_copy_play_opens_at_once() {
    for k in [StripKey::Copy, StripKey::Save] {
        let (mut rig, _) = sped(1);
        let (_, _, timed) = video_keep(&rig.on(key(k))).unwrap();
        rig.on(In::Kept(Outcome::Done));
        let into = timed.unwrap().into;
        assert_eq!(rig.on(key(StripKey::Play)), [Out::Play(into)], "{k:?}");
    }
}

#[test]
fn each_speed_has_its_own_copy_and_1x_plays_the_recording() {
    let (mut rig, video) = sped(1);
    let twice = played(&mut rig);
    rig.on(key(StripKey::Speed));
    let four = played(&mut rig);
    assert_ne!(twice, four);
    rig.on(key(StripKey::Speed));
    assert_eq!(rig.on(key(StripKey::Play)), [Out::Play(video)]);
    rig.on(key(StripKey::Speed));
    assert_eq!(rig.on(key(StripKey::Play)), [Out::Play(twice)]);
}

#[test]
fn a_failed_speed_job_notes_the_strip_stays_and_the_next_play_tries_again() {
    for note in [SnipNote::SpeedFailed, SnipNote::DiskLow] {
        let (mut rig, video) = sped(1);
        let (_, first) = retime(&rig.on(key(StripKey::Play))).unwrap();
        assert_eq!(rig.on(In::Kept(Outcome::Failed(note))), [Out::Note(note)]);
        assert_eq!(rig.stage(), &strip(video, speed(2), None));
        let (_, again) = retime(&rig.on(key(StripKey::Play))).unwrap();
        assert_ne!(first.into, again.into, "a copy not made is not reused");
    }
}

#[test]
fn a_cancelled_save_at_a_speed_leaves_no_copy_to_reuse() {
    let (mut rig, _) = sped(1);
    rig.on(key(StripKey::Save));
    rig.on(In::Kept(Outcome::Cancelled));
    assert!(retime(&rig.on(key(StripKey::Play))).is_some());
}

#[test]
fn an_end_while_plays_copy_is_made_deletes_it_and_plays_nothing() {
    let (mut rig, video) = sped(1);
    let (_, timed) = retime(&rig.on(key(StripKey::Play))).unwrap();
    let outs = rig.on(In::Quit);
    assert!(outs.contains(&Out::Delete(video)) && outs.contains(&Out::Delete(timed.into)));
    assert!(ended(&outs));
    assert_eq!(
        rig.on(In::Kept(Outcome::Done)),
        [],
        "a late answer plays nothing"
    );
}

#[test]
fn plays_copy_goes_at_the_end_unless_a_copy_put_it_on_the_clipboard() {
    for (copy_it, gone) in [(false, true), (true, false)] {
        let (mut rig, video) = sped(1);
        let copy = played(&mut rig);
        if copy_it {
            rig.on(key(StripKey::Copy));
            rig.on(In::Kept(Outcome::Done));
        } else {
            rig.on(key(StripKey::Close));
        }
        let outs = rig.on(if copy_it {
            key(StripKey::Close)
        } else {
            In::Ask(AskKey::Drop)
        });
        assert!(ended(&outs) && outs.contains(&Out::Delete(video)));
        assert_eq!(outs.contains(&Out::Delete(copy)), gone, "copied {copy_it}");
    }
}

#[test]
fn the_key_that_waits_for_its_copy_at_a_speed_is_told_from_the_writing_to_the_answer() {
    for k in [StripKey::Play, StripKey::Copy, StripKey::Save] {
        let (mut rig, _) = sped(1);
        rig.on(In::Timing);
        assert_eq!(rig.flow.waits(), None, "nothing asked");
        rig.on(key(k));
        assert_eq!(
            rig.flow.waits(),
            None,
            "{k:?} before its writing, as in Save's dialog"
        );
        rig.on(In::Timing);
        assert_eq!(rig.flow.waits(), Some(k), "{k:?}");
        rig.on(In::Kept(Outcome::Done));
        assert_eq!(rig.flow.waits(), None, "{k:?} answered");
        rig.on(key(k));
        assert_eq!(rig.flow.waits(), None, "{k:?} reuses the copy");
    }
    let (mut rig, _) = sped(0);
    rig.on(key(StripKey::Copy));
    rig.on(In::Timing);
    assert_eq!(rig.flow.waits(), None, "1× writes no copy");
}
