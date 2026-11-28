//! A recorded video's strip (ADR-0060): Play plays it, Speed steps 1×, 2×, 4× and back, Copy and Save keep it at that
//! speed with a tick while the strip stays, Edit does nothing, and Close asks when it was not kept (edges 6 and 7).

use super::*;
use kx_module_api::ports::SpeedAsk;

/// The strip of recorded `video` at `speed`, with the tick of `done`.
pub(super) fn strip(video: Video, speed: Speed, done: Option<Keep>) -> Stage {
    Stage::Strip(Strip {
        part: part(),
        home: home(),
        held: reel(video, speed),
        done,
        frame: None,
    })
}

/// The video Copy or Save `outs` asks for: Copy or Save, the recording, and its re-timed copy, if any.
pub(super) fn video_keep(outs: &[Out]) -> Option<(Keep, Video, Option<Timed>)> {
    outs.iter().find_map(|o| match o {
        Out::Keep {
            keep,
            what: What::Video { video, timed },
        } => Some((*keep, *video, *timed)),
        _ => None,
    })
}

/// A strip key.
pub(super) fn key(key: StripKey) -> In {
    In::Strip(key)
}

#[test]
fn edge_6_a_recording_that_ends_by_itself_shows_the_strip_stop_shows() {
    for end in [RecordEnd::Stopped, RecordEnd::Longest, RecordEnd::Full] {
        let mut rig = Rig::new();
        let video = rig.recording();
        rig.on(In::Note {
            video,
            note: done(end),
        });
        assert_eq!(rig.stage(), &strip(video, Speed::PLAIN, None), "{end:?}");
    }
}

#[test]
fn edit_does_nothing_on_a_video() {
    let mut rig = Rig::new();
    let video = rig.stopped();
    assert_eq!(rig.on(key(StripKey::Edit)), []);
    assert_eq!(rig.stage(), &strip(video, Speed::PLAIN, None));
}

#[test]
fn play_plays_the_recording_and_the_strip_stays() {
    let mut rig = Rig::new();
    let video = rig.stopped();
    assert_eq!(rig.on(key(StripKey::Play)), [Out::Play(video)]);
    assert_eq!(rig.stage(), &strip(video, Speed::PLAIN, None));
    assert!(!rig.flow.asking());
}

#[test]
fn play_and_speed_do_nothing_on_a_picture_or_before_recording() {
    let taken: [fn(&mut Rig); 2] = [Rig::picture, Rig::ready];
    for take in taken {
        let mut rig = Rig::new();
        take(&mut rig);
        let before = rig.stage().clone();
        for k in [StripKey::Play, StripKey::Speed] {
            assert_eq!(rig.on(key(k)), [], "{k:?}");
        }
        assert_eq!(rig.stage(), &before);
    }
}

#[test]
fn speed_steps_through_the_speeds_and_back_to_as_recorded() {
    let mut rig = Rig::new();
    let video = rig.stopped();
    let faster = SpeedAsk::SPEEDS[1..].iter().map(|&t| speed(t));
    for want in faster.chain([Speed::PLAIN]) {
        assert_eq!(rig.on(key(StripKey::Speed)), []);
        assert_eq!(rig.stage(), &strip(video, want, None));
    }
}

#[test]
fn speed_retimes_into_a_new_video_that_a_copy_keeps() {
    let mut rig = Rig::new();
    let video = rig.stopped();
    rig.on(key(StripKey::Speed));
    let (_, from, timed) = video_keep(&rig.on(key(StripKey::Copy))).unwrap();
    let timed = timed.unwrap();
    assert_eq!((from, timed.speed), (video, speed(2)));
    assert_ne!(timed.into, video);
    rig.on(In::Kept(Outcome::Done));
    let outs = rig.on(key(StripKey::Close));
    assert_eq!(outs.first(), Some(&Out::Delete(video)), "the copy stays");
    assert!(ended(&outs) && !outs.contains(&Out::Delete(timed.into)));
}

#[test]
fn a_kept_video_ticks_while_its_strip_stays() {
    for (k, keep) in [(StripKey::Copy, Keep::Copy), (StripKey::Save, Keep::Save)] {
        let mut rig = Rig::new();
        let video = rig.stopped();
        rig.on(key(k));
        assert_eq!(rig.on(In::Kept(Outcome::Done)), []);
        assert_eq!(rig.stage(), &strip(video, Speed::PLAIN, Some(keep)));
        assert_eq!(rig.wait(rig.shown()), []);
        assert_eq!(rig.stage(), &strip(video, Speed::PLAIN, None));
    }
}

#[test]
fn edge_7_a_copy_at_2x_then_speed_4x_copies_again_at_4x() {
    let mut rig = Rig::new();
    let video = rig.stopped();
    rig.on(key(StripKey::Speed));
    let (_, _, first) = video_keep(&rig.on(key(StripKey::Copy))).unwrap();
    rig.on(In::Kept(Outcome::Done));
    assert_eq!(
        rig.on(key(StripKey::Speed)),
        [],
        "it acts while the tick shows"
    );
    assert_eq!(rig.stage(), &strip(video, speed(4), None));
    assert_eq!(rig.flow.deadline(), None, "the change took the tick");
    let (_, _, again) = video_keep(&rig.on(key(StripKey::Copy))).unwrap();
    let (first, again) = (first.unwrap(), again.unwrap());
    assert_eq!((first.speed, again.speed), (speed(2), speed(4)));
    assert_ne!(first.into, again.into, "each speed its own copy");
}

#[test]
fn a_speed_change_makes_it_unkept_and_a_full_round_plays_as_recorded() {
    let mut rig = Rig::new();
    let video = rig.stopped();
    rig.on(key(StripKey::Copy));
    rig.on(In::Kept(Outcome::Done));
    rig.on(key(StripKey::Speed));
    rig.on(key(StripKey::Close));
    assert!(rig.flow.asking());
    rig.on(In::Ask(AskKey::Back));
    rig.on(key(StripKey::Speed));
    rig.on(key(StripKey::Speed));
    assert_eq!(rig.stage(), &strip(video, Speed::PLAIN, None));
    let outs = rig.on(key(StripKey::Save));
    assert_eq!(video_keep(&outs), Some((Keep::Save, video, None)));
}

#[test]
fn a_copied_video_stays_in_unsaved_and_a_saved_one_goes_at_the_end() {
    for (k, gone) in [(StripKey::Copy, false), (StripKey::Save, true)] {
        let mut rig = Rig::new();
        let video = rig.stopped();
        rig.on(key(k));
        rig.on(In::Kept(Outcome::Done));
        let outs = rig.on(key(StripKey::Close));
        assert_eq!(outs.contains(&Out::Delete(video)), gone, "{k:?}");
        assert_eq!(outs.last(), Some(&Out::End));
    }
}

#[test]
fn the_questions_save_keeps_a_video_at_its_speed_then_ends() {
    let mut rig = Rig::new();
    let video = rig.stopped();
    rig.on(key(StripKey::Speed));
    rig.on(key(StripKey::Speed));
    rig.on(key(StripKey::Close));
    let (keep, _, timed) = video_keep(&rig.on(In::Ask(AskKey::Save))).unwrap();
    let timed = timed.unwrap();
    assert_eq!((keep, timed.speed), (Keep::Save, speed(4)));
    assert_eq!(rig.on(In::Kept(Outcome::Done)), []);
    let outs = rig.wait(rig.shown());
    assert_eq!(outs[..2], [Out::Delete(video), Out::Delete(timed.into)]);
    assert!(ended(&outs));
}

#[test]
fn while_the_questions_save_ticks_only_close_acts_and_the_snip_ends() {
    let mut rig = Rig::new();
    let video = rig.stopped();
    rig.on(key(StripKey::Close));
    rig.on(In::Ask(AskKey::Save));
    rig.on(In::Kept(Outcome::Done));
    for k in [
        StripKey::Speed,
        StripKey::Play,
        StripKey::Copy,
        StripKey::Save,
    ] {
        assert_eq!(rig.on(key(k)), [], "{k:?}");
    }
    assert_eq!(rig.stage(), &strip(video, Speed::PLAIN, Some(Keep::Save)));
    assert!(
        ended(&rig.wait(rig.shown())),
        "the snip asked to close ends"
    );
}

#[test]
fn a_length_at_a_speed_rounds_up_so_a_last_part_still_shows() {
    let video = Rig::new().stopped();
    let clip = Recorded {
        video,
        length_ms: 1_001,
    };
    let at = |speed| Reel { clip, speed }.length_ms();
    assert_eq!(at(Speed::PLAIN), 1_001);
    assert_eq!((at(speed(2)), at(speed(4))), (501, 251));
}

#[test]
fn keys_wait_while_a_copy_or_save_runs() {
    let mut rig = Rig::new();
    let video = rig.stopped();
    rig.on(key(StripKey::Save));
    for k in [
        StripKey::Speed,
        StripKey::Play,
        StripKey::Copy,
        StripKey::Close,
    ] {
        assert_eq!(rig.on(key(k)), [], "{k:?}");
    }
    assert_eq!(rig.stage(), &strip(video, Speed::PLAIN, None));
}

#[test]
fn the_key_with_an_unkept_video_asks_and_dont_save_deletes_it() {
    let mut rig = Rig::new();
    let video = rig.stopped();
    assert_eq!(rig.on(In::Key), []);
    assert!(rig.flow.asking());
    let outs = rig.on(In::Ask(AskKey::Drop));
    assert!(
        outs.contains(&Out::Delete(video)) && ended(&outs),
        "{outs:?}"
    );
    assert!(!outs.contains(&Out::Lit(SnipLit::On)), "no new snip");
}

#[test]
fn a_late_editor_change_leaves_a_saved_videos_strip_kept() {
    let mut rig = Rig::new();
    let video = rig.stopped();
    rig.on(key(StripKey::Save));
    rig.on(In::Kept(Outcome::Done));
    assert_eq!(rig.on(In::Changed), [], "only the editor's picture changes");
    assert_eq!(rig.stage(), &strip(video, Speed::PLAIN, Some(Keep::Save)));
    assert_eq!(rig.on(In::Key).last(), Some(&Out::End), "it ends unasked");
}
