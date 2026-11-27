//! How a run ends: goodbye and the end of input stop it and keep the video, the longest length ends it by itself and a
//! late Cancel or switch keeps it (edge 9), a start fault leaves no file (edges 11 and 14), and a panic inside deletes
//! the video (edge 13) while the job goes on.

use super::super::job::Job;
use super::super::{Event, Notes};
use super::*;
use kx_ipc::IpcError;
use std::sync::mpsc;

impl Notes for Vec<RecordNote> {
    fn tell(&mut self, note: RecordNote) -> Result<(), IpcError> {
        self.push(note);
        Ok(())
    }
}

/// A run stopped by `bye` (true) or by the end of input keeps its video and tells its end before the job exits.
fn stopped_by(bye: bool) {
    let dir = Dir::new();
    let file = dir.file("snip 1.mp4");
    let studio = held(script());
    let mut host = Host::start(&studio);
    host.send(&start(&file));
    studio.grant(1);
    assert_eq!(host.next(), time(0));
    studio.free();
    let (ended, told) = host.finish(bye);
    assert_eq!(ended, Ok(()));
    assert!(
        matches!(
            told.as_slice(),
            [
                ..,
                RecordNote::End {
                    id: ID,
                    end: RecordEnd::Stopped,
                    ..
                }
            ]
        ),
        "{told:?}"
    );
    assert_eq!(fs::read(&file).unwrap(), ENDED);
}

#[test]
fn goodbye_stops_a_run_and_keeps_its_video() {
    stopped_by(true);
}

#[test]
fn the_end_of_input_stops_a_run_and_keeps_its_video() {
    stopped_by(false);
}

#[test]
fn the_longest_length_ends_a_run_by_itself_and_a_late_cancel_keeps_it() {
    let dir = Dir::new();
    let file = dir.file("snip 1.mp4");
    let (studio, _) = FakeStudio::new(script(), Vec::new());
    let mut host = Host::start(&studio);
    host.send(&start(&file));
    let seconds: Vec<RecordNote> = (0..60).map(|s| time(s * 1_000)).collect();
    let told: Vec<RecordNote> = (0..=60).map(|_| host.next()).collect();
    assert_eq!(told[..60], seconds);
    assert_eq!(told[60], end(60_000, RecordEnd::Longest));
    host.send(&RecordAsk::Cancel { id: ID });
    host.send(&RecordAsk::Stop { id: ID });
    assert_eq!(host.barrier(), []);
    assert_eq!(fs::read(&file).unwrap(), ENDED);
}

/// A change to the script, and the start fault it brings.
type Case = (fn(&mut Script), RecordFault);

#[test]
fn a_start_fault_is_told_and_leaves_no_file() {
    let refused: [Case; 3] = [
        (
            |s| s.ready = Err(RecordFault::NoVideo),
            RecordFault::NoVideo,
        ),
        (
            |s| s.mic = Voice::refused(RecordFault::MicRefused),
            RecordFault::MicRefused,
        ),
        (
            |s| s.room = vec![(Duration::ZERO, Some(1))],
            RecordFault::Full,
        ),
    ];
    for (change, fault) in refused {
        let dir = Dir::new();
        let file = dir.file("snip 1.mp4");
        let mut s = script();
        change(&mut s);
        let (studio, _) = FakeStudio::new(s, Vec::new());
        let mut host = Host::start(&studio);
        host.send(&start(&file));
        assert_eq!(host.next(), RecordNote::Fault { id: ID, fault });
        assert_eq!(host.barrier(), []);
        assert!(!file.exists(), "{fault:?}");
    }
}

#[test]
fn a_run_that_panics_is_told_failed_its_video_deleted_and_the_job_goes_on() {
    let dir = Dir::new();
    let file = dir.file("snip 1.mp4");
    let mut s = script();
    s.sink_panics_at = Some(1);
    let (studio, _) = FakeStudio::new(s, Vec::new());
    let mut host = Host::start(&studio);
    host.send(&start(&file));
    assert_eq!(host.next(), time(0));
    let fault = RecordFault::Failed;
    assert_eq!(host.next(), RecordNote::Fault { id: ID, fault });
    assert!(!file.exists());
    studio.world().script.sink_panics_at = None;
    host.send(&start_as(ID + 1, &file));
    assert_eq!(host.next(), RecordNote::Time { id: ID + 1, ms: 0 });
}

#[test]
fn a_switch_between_a_runs_end_and_its_done_tells_nothing_and_the_end_keeps_the_video() {
    let dir = Dir::new();
    let file = dir.file("snip 1.mp4");
    let (studio, _) = FakeStudio::new(script(), Vec::new());
    let (tx, events) = mpsc::channel();
    let (mut job, mut told) = (Job::new(studio, tx), Vec::new());
    job.take(Event::Ask(start(&file)), &mut told).unwrap();
    job.take(Event::Ask(RecordAsk::Stop { id: ID }), &mut told)
        .unwrap();
    // The run dropped its orders before its `Done` was sent, and the job has not taken that `Done` yet.
    let done = loop {
        match events.recv_timeout(WAIT).expect("the run's end") {
            Event::Note(note) => told.push(note),
            done @ Event::Done(..) => break done,
            _ => {}
        }
    };
    let late = RecordAsk::Mic { id: ID, on: false };
    job.take(Event::Ask(late), &mut told).unwrap();
    job.take(done, &mut told).unwrap();
    let ended = |n: &RecordNote| {
        matches!(
            n,
            RecordNote::End {
                id: ID,
                end: RecordEnd::Stopped,
                ..
            }
        )
    };
    assert!(told.last().is_some_and(ended), "{told:?}");
    assert!(!told.iter().any(|n| matches!(n, RecordNote::Fault { .. })));
    assert_eq!(fs::read(&file).unwrap(), ENDED);
}
