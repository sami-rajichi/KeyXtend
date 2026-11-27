//! Re-timed copies: each in a record job of its own into a new video of the unsaved part, refused as the port says
//! before any job, the job's fault or a lost job deleting the copy, and the job told goodbye once it answered.

use super::*;

/// A recording held in the unsaved part, as the fake folder keeps it.
fn recorded(rig: &Rig, name: &str) -> SnipFile {
    let video = SnipFile::new(unsaved().expose().join(name), FileKind::Mp4).unwrap();
    rig.folder.put(
        video.clone(),
        Stored::Video {
            length_ms: LENGTH_MS,
        },
    );
    video
}

/// A 2x copy of `from` at the least floor and cap.
fn speed(from: SnipFile) -> SpeedAsk {
    SpeedAsk {
        from,
        speed: Speed::new(2).unwrap(),
        min_free_mb: *RecordAsk::MIN_FREE_MB.start(),
        job_mb: *RecordAsk::JOB_MB.end(),
        waits: waits(),
    }
}

/// A copy's end.
const fn copied(id: u32) -> Note {
    let (length_ms, end) = (LENGTH_MS / 2, End::Stopped);
    Note::End { id, length_ms, end }
}

#[test]
fn a_copy_runs_a_job_of_its_own_into_a_new_video_and_says_goodbye_once_done() {
    let rig = Rig::new();
    let from = recorded(&rig, "a.mp4");
    rig.script.act(RecordAct::Say(vec![copied]));
    let to = rig.recorder.speed(&speed(from.clone())).unwrap();
    assert!(to.lies_in(&unsaved()) && to != from && to.kind() == FileKind::Mp4);
    assert_eq!(
        rig.script.launches(),
        [Launch::record(speed(from.clone()).job_mb)]
    );
    let path = |f: &SnipFile| f.expose().to_str().unwrap().to_owned().into();
    let want = Ask::Speed {
        id: 1,
        from: path(&from),
        speed: 2,
        to: path(&to),
    };
    assert_eq!(rig.asks(), [want]);
    assert!(rig.folder.removes().is_empty());
}

#[test]
fn a_copy_is_refused_as_the_port_says_before_any_job() {
    let rig = Rig::new();
    let from = recorded(&rig, "a.mp4");
    let picture = SnipFile::new(unsaved().expose().join("a.png"), FileKind::Png).unwrap();
    let elsewhere = SnipFile::new(PathBuf::from(SNIPS).join("b.mp4"), FileKind::Mp4).unwrap();
    rig.folder
        .put(elsewhere.clone(), Stored::Video { length_ms: 1 });
    let gone = SnipFile::new(unsaved().expose().join("c.mp4"), FileKind::Mp4).unwrap();
    let big = SpeedAsk {
        job_mb: *RecordAsk::JOB_MB.end() + 1,
        ..speed(from.clone())
    };
    let refusals = [
        (speed(picture), RecordError::BadPath),
        (big, RecordError::Range("job_mb")),
        (speed(elsewhere), RecordError::BadPath),
        (speed(gone), RecordError::BadPath),
    ];
    for (asked, error) in &refusals {
        assert_eq!(rig.recorder.speed(asked), Err(*error));
    }
    let floor = u64::from(speed(from.clone()).min_free_mb) * MB;
    rig.folder.set_free(Ok(floor - 1));
    assert_eq!(rig.recorder.speed(&speed(from)), Err(RecordError::Full));
    assert_eq!(rig.script.tries(), 0, "no job for a refused copy");
}

#[test]
fn a_copys_fault_or_lost_job_deletes_the_copy() {
    let rig = Rig::new();
    let from = recorded(&rig, "a.mp4");
    let faults: [(Says, RecordError); 3] = [
        (|id| fault(id, RecordFault::NoVideo), RecordError::NoVideo),
        (|id| fault(id, RecordFault::Full), RecordError::Full),
        (failed, RecordError::Failed),
    ];
    for (note, error) in faults {
        rig.script.act(RecordAct::Say(vec![note]));
        assert_eq!(rig.recorder.speed(&speed(from.clone())), Err(error));
    }
    let removed = rig.folder.removes();
    assert_eq!(removed.len(), faults.len());
    assert!(removed.iter().all(|f| *f != from), "the source stays");
    let (calls, asked) = mpsc::channel();
    rig.script.tell(calls);
    let recorder = Arc::clone(&rig.recorder);
    let ask = speed(from);
    let copying = std::thread::spawn(move || recorder.speed(&ask));
    assert!(matches!(sent(&asked), Ask::Speed { .. }));
    rig.script.quit(DIED);
    assert_eq!(copying.join().unwrap(), Err(RecordError::Gone));
    assert_eq!(rig.folder.removes().len(), faults.len() + 1);
}

#[test]
fn a_copy_its_job_never_ends_is_gone_at_its_deadline_and_deleted() {
    let rig = Rig::new();
    let from = recorded(&rig, "a.mp4");
    let (calls, asked) = mpsc::channel();
    rig.script.tell(calls);
    let recorder = Arc::clone(&rig.recorder);
    let ask = speed(from);
    let copying = std::thread::spawn(move || recorder.speed(&ask));
    assert!(matches!(sent(&asked), Ask::Speed { .. }));
    rig.clock.advance(deadline(&waits(), 0));
    rig.script.say(Ok(second(1)));
    assert_eq!(copying.join().unwrap(), Err(RecordError::Gone));
    assert_eq!(rig.folder.removes().len(), 1);
}

#[test]
fn a_failed_copy_the_folder_still_held_is_deleted_again_once_its_job_ended() {
    let rig = Rig::new();
    let from = recorded(&rig, "a.mp4");
    rig.folder.refuse_removes(SaveError::Io, 1);
    rig.script.act(RecordAct::Say(vec![failed]));
    let copied = rig.recorder.speed(&speed(from));
    assert_eq!(copied, Err(RecordError::Failed));
    let removed = rig.folder.removes();
    assert_eq!(removed.len(), 2, "tried again: {removed:?}");
    assert_eq!(removed[0], removed[1]);
}
