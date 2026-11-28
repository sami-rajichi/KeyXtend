//! A run's start: each start fault (edges 11 and 14) ends it and leaves no file, a sound left off is never opened, and
//! an existing file is never written over.

use super::*;

/// A change to the script or the plan, and the start fault it brings.
type Case = (fn(&mut Script, &mut Plan), RecordFault);

#[test]
fn each_start_fault_ends_the_start_and_leaves_no_file() {
    let cases: [Case; 7] = [
        (
            |s, _| s.room = vec![(Duration::ZERO, Some(LOW))],
            RecordFault::Full,
        ),
        (
            |s, _| s.room = vec![(Duration::ZERO, None)],
            RecordFault::Failed,
        ),
        (
            |s, _| s.ready = Err(RecordFault::NoVideo),
            RecordFault::NoVideo,
        ),
        (
            |s, _| s.mic = Voice::refused(RecordFault::NoMic),
            RecordFault::NoMic,
        ),
        (
            |s, _| s.mic = Voice::refused(RecordFault::MicRefused),
            RecordFault::MicRefused,
        ),
        (
            |s, _| s.sound = Voice::refused(RecordFault::NoSound),
            RecordFault::NoSound,
        ),
        (|s, _| s.frames_from = secs(5.0), RecordFault::Failed),
    ];
    for (n, (change, want)) in cases.into_iter().enumerate() {
        let temp = Temp::new();
        let (mut s, mut p) = (script(), plan(temp.path()));
        change(&mut s, &mut p);
        let (notes, _) = run(s, &p, vec![(secs(1.0), Order::Stop)]);
        assert_eq!(notes, [fault(want)], "case {n}");
        assert!(!p.file.exists(), "case {n}");
    }
}

#[test]
fn a_sound_left_off_is_never_opened() {
    let temp = Temp::new();
    let (mut s, mut p) = (script(), plan(temp.path()));
    (s.mic, s.sound) = (
        Voice::refused(RecordFault::NoMic),
        Voice::refused(RecordFault::NoSound),
    );
    (p.mic, p.sound) = (false, false);
    let (notes, log) = run(s, &p, vec![(secs(0.5), Order::Stop)]);
    assert_eq!(notes.last(), Some(&end(500, RecordEnd::Stopped)));
    assert!(log.sound.iter().all(|s| *s == 0));
}

#[test]
fn an_existing_file_is_never_written_over() {
    let temp = Temp::new();
    let p = plan(temp.path());
    fs::write(&p.file, b"keep").unwrap();
    let (notes, _) = run(script(), &p, vec![(secs(1.0), Order::Stop)]);
    assert_eq!(notes, [fault(RecordFault::Failed)]);
    assert_eq!(fs::read(&p.file).unwrap(), b"keep");
}
