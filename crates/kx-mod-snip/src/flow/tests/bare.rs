//! A capture window that cannot be kept out of captures hides while a video records, so the ui's `Bare` ends a countdown
//! and stops a recording as Stop does, keeping the video, with a note saying why; a picture's snip goes on.

use super::*;

#[test]
fn bare_ends_the_countdown_with_no_file_and_says_why() {
    let mut rig = Rig::new();
    rig.ready();
    rig.on(In::Strip(StripKey::Record));
    assert!(matches!(rig.stage(), Stage::Count(_)));
    let outs = rig.on(In::Bare);
    assert!(ended(&outs) && !kept_any(&outs), "{outs:?}");
    assert_eq!(outs.first(), Some(&Out::Note(SnipNote::Bare)), "{outs:?}");
    assert_eq!(rig.stage(), &Stage::Idle);
}

#[test]
fn bare_stops_the_recording_as_stop_does_keeps_its_video_and_says_why() {
    let mut rig = Rig::new();
    let video = rig.recording();
    let stop = Out::Run {
        video,
        call: Call::Stop,
    };
    assert_eq!(rig.on(In::Bare), [Out::Note(SnipNote::Bare), stop]);
    assert_eq!(rig.on(In::Bare), [], "once, as Stop");
    rig.on(In::Note {
        video,
        note: done(RecordEnd::Stopped),
    });
    let Stage::Strip(strip) = rig.stage() else {
        panic!("the video's strip shows");
    };
    assert_eq!(strip.held, reel(video, Speed::PLAIN));
}

#[test]
fn bare_leaves_a_snip_with_no_video_running_as_it_was() {
    let mut rig = Rig::new();
    rig.on(In::Key);
    assert_eq!(rig.on(In::Bare), [], "waiting for the keyboard");
    rig.on(In::Aside);
    rig.on(In::Frozen(Ok(frozen())));
    assert_eq!(rig.on(In::Bare), [], "picking");
    rig.take();
    let before = rig.stage().clone();
    assert_eq!(rig.on(In::Bare), [], "a picture's strip");
    assert_eq!(rig.stage(), &before);
    rig.on(In::Strip(StripKey::Edit));
    assert_eq!(rig.on(In::Bare), [], "the editor");
}
