//! A recording's faults and late notes: a fault notes and ends while the host deletes the broken video, and a note of
//! a run the flow left cancels that run or deletes its kept video.

use super::*;

fn fault(rig: &mut Rig, video: Video, error: RecordError) -> Vec<Out> {
    rig.on(In::Note {
        video,
        note: RecordNote::Fault(error),
    })
}

/// Record pressed and counted down, with no note heard yet.
fn starting(rig: &mut Rig) -> Video {
    rig.ready();
    let mut outs = rig.on(In::Strip(StripKey::Record));
    outs.extend(rig.wait(rig.countdown()));
    recorded(&outs).unwrap()
}

#[test]
fn a_fault_mid_run_notes_and_ends() {
    let mut rig = Rig::new();
    let video = rig.recording();
    let outs = fault(&mut rig, video, RecordError::Full);
    assert!(outs.contains(&Out::Note(SnipNote::RecordFailed)));
    assert!(ended(&outs));
    assert!(!outs.iter().any(|o| matches!(o, Out::Run { .. })));
    assert!(!outs.contains(&Out::Delete(video)));
    assert_eq!(rig.stage(), &Stage::Idle);
}

#[test]
fn a_start_fault_says_why() {
    for (error, note) in [
        (RecordError::NoVideo, SnipNote::NoVideo),
        (RecordError::Full, SnipNote::DiskLow),
        (RecordError::Gone, SnipNote::RecordFailed),
    ] {
        let mut rig = Rig::new();
        let video = starting(&mut rig);
        let outs = fault(&mut rig, video, error);
        assert!(outs.contains(&Out::Note(note)), "{error:?}");
        assert!(ended(&outs), "{error:?}");
    }
}

#[test]
fn stop_before_the_start_is_heard_still_stops() {
    let mut rig = Rig::new();
    let video = starting(&mut rig);
    let stop = Out::Run {
        video,
        call: Call::Stop,
    };
    assert_eq!(rig.on(In::Record(RecKey::Stop)), [stop]);
}

#[test]
fn a_late_end_of_a_cancelled_run_deletes_its_video() {
    let mut rig = Rig::new();
    let video = rig.recording();
    rig.on(In::Record(RecKey::Cancel));
    let late = In::Note {
        video,
        note: done(RecordEnd::Stopped),
    };
    assert_eq!(rig.on(late), [Out::Delete(video)]);
}

#[test]
fn a_late_time_of_a_left_run_cancels_it() {
    let mut rig = Rig::new();
    let video = rig.recording();
    rig.on(In::Quit);
    let late = In::Note {
        video,
        note: RecordNote::Time { ms: 1_000 },
    };
    let cancel = Out::Run {
        video,
        call: Call::Cancel,
    };
    assert_eq!(rig.on(late), [cancel]);
}

#[test]
fn a_late_fault_does_nothing() {
    let mut rig = Rig::new();
    let video = rig.recording();
    rig.on(In::Record(RecKey::Cancel));
    assert_eq!(fault(&mut rig, video, RecordError::Gone), []);
}

#[test]
fn an_old_runs_note_leaves_the_new_recording_alone() {
    let mut rig = Rig::new();
    let old = rig.recording();
    rig.on(In::Record(RecKey::Cancel));
    let new = rig.recording();
    assert_ne!(old, new);
    let late = In::Note {
        video: old,
        note: done(RecordEnd::Stopped),
    };
    assert_eq!(rig.on(late), [Out::Delete(old)]);
    assert!(rec_of(&rig, new).live);
}

/// The running recording of `video`, as the stage shows it.
fn rec_of(rig: &Rig, video: Video) -> Rec {
    match rig.stage() {
        Stage::Record(rec) if rec.video == video => rec.clone(),
        other => panic!("not recording {video:?}: {other:?}"),
    }
}

#[test]
fn notes_outside_a_recording_change_nothing() {
    let mut rig = Rig::new();
    rig.picture();
    let before = rig.stage().clone();
    let stray = In::Note {
        video: Video(77),
        note: RecordNote::Fault(RecordError::Gone),
    };
    assert_eq!(rig.on(stray), []);
    assert_eq!(rig.stage(), &before);
}
