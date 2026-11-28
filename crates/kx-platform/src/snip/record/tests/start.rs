//! A start: its own record job capped at `job_mb` with the exact Start frame, the port's refusals before any job, a
//! start's fault as its error with the video gone, and notes in the job's order until Stop keeps the video.

use super::*;

#[test]
fn a_start_runs_a_record_job_of_its_own_capped_at_job_mb_into_a_new_video_of_the_unsaved_part() {
    let rig = Rig::new();
    let (run, _heard) = rig.live();
    assert!(run.file().lies_in(&unsaved()) && run.file().kind() == FileKind::Mp4);
    assert_eq!(rig.script.launches(), [Launch::record(ask().job_mb)]);
    let path = run.file().expose().to_str().unwrap().to_owned();
    let area = Area {
        x: PART.x,
        y: PART.y,
        w: PART.w,
        h: PART.h,
    };
    let want = Ask::Start {
        id: 1,
        area,
        fps: ask().fps,
        mic: false,
        sound: true,
        max_min: ask().max_min,
        min_free_mb: ask().min_free_mb,
        file: path.into(),
    };
    assert_eq!(rig.asks(), [want]);
    let first = run.file().clone();
    rig.close(run);
    let (second, _heard) = rig.live();
    assert_ne!(
        second.file(),
        &first,
        "each its own video, named in the same second"
    );
    assert_eq!(rig.script.spawns(), 2, "each its own job");
}

#[test]
fn a_start_is_refused_in_the_ports_order_before_any_job_starts() {
    let rig = Rig::new();
    let other = MicSeal::new().pass();
    let floor = u64::from(ask().min_free_mb) * MB;
    rig.folder.set_free(Ok(floor - 1));
    let refusals = [
        (RecordAsk { fps: 25, ..ask() }, RecordError::Range("fps")),
        (
            RecordAsk {
                mic: Some(other),
                ..ask()
            },
            RecordError::BadPass,
        ),
        (ask(), RecordError::Full),
    ];
    for (asked, error) in refusals {
        assert_eq!(rig.start(asked, Vec::new()).0.err(), Some(error));
    }
    let clock = Hands::default();
    let hidden = FakeSnipFolder::hidden(Arc::new(clock.clone()));
    let hidden = WorkerRecorder::new(
        rig.script.clone(),
        Arc::new(clock),
        MicSeal::new(),
        Arc::new(hidden),
        Arc::new(Facts(READY)),
    );
    let refused = hidden.start(ask(), ear().0).err();
    assert_eq!(refused, Some(RecordError::NoFolder), "a hidden run");
    assert_eq!(rig.script.tries(), 0, "no job for a refused start");
}

#[test]
fn a_fault_before_the_first_time_is_the_starts_error_telling_nothing_and_leaving_no_video() {
    let rig = Rig::new();
    let faults: [(Says, RecordError); 6] = [
        (|id| fault(id, RecordFault::NoVideo), RecordError::NoVideo),
        (|id| fault(id, RecordFault::NoMic), RecordError::NoMic),
        (
            |id| fault(id, RecordFault::MicRefused),
            RecordError::MicRefused,
        ),
        (|id| fault(id, RecordFault::NoSound), RecordError::NoSound),
        (|id| fault(id, RecordFault::Full), RecordError::Full),
        (failed, RecordError::Failed),
    ];
    for (note, error) in faults {
        let (run, heard) = rig.start(ask(), vec![note]);
        assert_eq!(run.err(), Some(error));
        assert!(
            heard.try_recv().is_err(),
            "a start's fault is its error alone"
        );
    }
    assert_eq!(
        rig.folder.removes().len(),
        faults.len(),
        "each video named goes"
    );
    assert!(rig.folder.files().is_empty());
}

#[test]
fn notes_reach_the_callback_in_the_jobs_order_and_stop_ends_with_the_video_kept() {
    let rig = Rig::new();
    let (run, heard) = rig.start(ask(), vec![started, second, mic_lost, second]);
    let mut run = run.unwrap();
    let want = [
        RecordNote::Time { ms: 0 },
        RecordNote::Time { ms: 1_000 },
        RecordNote::Lost(Source::Mic),
        RecordNote::Time { ms: 1_000 },
    ];
    for note in want {
        assert_eq!(next(&heard), note);
    }
    rig.script.act(RecordAct::Say(vec![stopped]));
    assert_eq!(run.stop(), Ok(()));
    let done = RecordDone {
        length_ms: LENGTH_MS,
        end: RecordEnd::Stopped,
    };
    assert_eq!(next(&heard), RecordNote::Done(done));
    assert_eq!(rig.asks().last(), Some(&Ask::Stop { id: 1 }));
    assert!(rig.folder.removes().is_empty(), "the video stays");
    assert_eq!(run.stop(), Err(RecordError::Gone), "it ended");
    run.cancel();
    assert!(
        rig.folder.removes().is_empty(),
        "a cancel after the end keeps it"
    );
    assert!(!rig.asks().contains(&Ask::Cancel { id: 1 }));
}
