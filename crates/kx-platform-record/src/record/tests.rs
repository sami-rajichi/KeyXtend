//! A run over the scripted studio, its clock jumping from slot to slot and its orders posted at set times: frames at
//! `fps` and the time once a second, Stop keeps, Cancel deletes, the longest length and the floor end and keep (edges 8
//! and 9), and a broken file is deleted. The start, the sound and the last note have their own files.

mod last;
mod sound;
mod start;

use super::*;
use crate::fake::{FakeStudio, Log, MIC, SOUND, Script, Tape, Voice};
use kx_ipc::{SNIP_FPS, SNIP_MIN_FREE_MB};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

/// The run's id.
const ID: u32 = 7;
/// The higher of `SNIP_FPS`.
const FPS: u8 = SNIP_FPS[1];
/// Room under the floor, in MiB.
const LOW: u64 = 100;
/// The floor asked, the bottom of `SNIP_MIN_FREE_MB`.
const FLOOR: u32 = *SNIP_MIN_FREE_MB.start();

/// A test's own folder under the temp folder, deleted with what it holds when dropped.
pub(crate) struct Temp(PathBuf);

impl Temp {
    pub(crate) fn new() -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("kx-record-{}-{n}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// `s` seconds, as orders and scripts time things.
pub(crate) fn secs(s: f64) -> Duration {
    Duration::from_secs_f64(s)
}

/// The most samples one sound write may hold: a source's queue, in stereo samples.
pub(crate) fn widest() -> usize {
    let queue_ms = u64::from(crate::table::table().unwrap().sound.queue_ms);
    let frames = queue_ms * u64::from(RATE) / MS_S;
    usize::try_from(frames).unwrap() * usize::from(CHANNELS)
}

/// A script where everything works, with an empty tape.
pub(crate) fn script() -> Script {
    Script::working(Tape {
        shape: Shape {
            width: 640,
            height: 360,
            fps: u32::from(FPS),
        },
        frames: 0,
        step: 1,
        sound: 0,
        level: 0,
        length_ms: 0,
    })
}

/// A one-minute recording of an odd-sized part with both sounds into `dir`.
fn plan(dir: &Path) -> Plan {
    Plan {
        id: ID,
        area: Area {
            x: -10,
            y: 20,
            w: 641,
            h: 361,
        },
        fps: FPS,
        mic: true,
        sound: true,
        max_min: 1,
        min_free_mb: FLOOR,
        file: dir.join("snip 1.mp4"),
    }
}

/// Runs `plan` over `script` with `orders` posted at their times; the notes told, the last one handed back after them,
/// and the fakes' log.
pub(crate) fn run(
    script: Script,
    plan: &Plan,
    orders: Vec<(Duration, Order)>,
) -> (Vec<RecordNote>, Log) {
    let (studio, rx) = FakeStudio::new(script, orders);
    let mut notes = Vec::new();
    let last = record(&studio, plan, &rx, &mut |n| notes.push(n));
    notes.extend(last);
    let log = studio.world().log.clone();
    (notes, log)
}

/// A frame's write that takes two slots, so the run falls behind.
fn slow() -> Duration {
    Duration::from_secs(2) / u32::from(FPS)
}

/// The times told, in ms.
fn times(notes: &[RecordNote]) -> Vec<u32> {
    let time = |n: &RecordNote| match n {
        RecordNote::Time { ms, .. } => Some(*ms),
        _ => None,
    };
    notes.iter().filter_map(time).collect()
}

/// The end told with its length.
fn end(length_ms: u32, end: RecordEnd) -> RecordNote {
    RecordNote::End {
        id: ID,
        length_ms,
        end,
    }
}

/// A fault on the run.
fn fault(fault: RecordFault) -> RecordNote {
    RecordNote::Fault { id: ID, fault }
}

#[test]
fn frames_come_at_fps_and_the_time_once_a_second() {
    let temp = Temp::new();
    let p = plan(temp.path());
    let (notes, log) = run(script(), &p, vec![(secs(2.5), Order::Stop)]);
    assert_eq!(times(&notes), [0, 1000, 2000]);
    assert_eq!(notes.last(), Some(&end(2500, RecordEnd::Stopped)));
    assert_eq!(log.frames.len(), 75);
    for (n, (slot, made)) in log.frames.iter().enumerate() {
        assert_eq!(*slot, u64::try_from(n).unwrap());
        assert_eq!(*made, Duration::from_secs(*slot) / u32::from(FPS));
    }
    assert!(log.finished);
    assert_eq!(fs::read(&p.file).unwrap(), b"ended");
}

#[test]
fn stop_keeps_the_video_with_its_sound_to_its_last_frame() {
    let temp = Temp::new();
    let (_, log) = run(script(), &plan(temp.path()), vec![(secs(1.0), Order::Stop)]);
    let per_frame = RATE / u32::from(FPS);
    let want = log.frames.len() * usize::try_from(per_frame * u32::from(CHANNELS)).unwrap();
    assert_eq!(log.sound.len(), want);
    assert_eq!(log.jumps, 0);
}

#[test]
fn cancel_deletes_the_video_and_tells_nothing_more() {
    let temp = Temp::new();
    let p = plan(temp.path());
    let (notes, log) = run(script(), &p, vec![(secs(1.2), Order::Cancel)]);
    assert_eq!(times(&notes), [0, 1000]);
    assert_eq!(notes.len(), 2, "{notes:?}");
    assert!(!log.finished && !p.file.exists());
}

#[test]
fn the_longest_length_ends_the_run_and_keeps_the_video() {
    let temp = Temp::new();
    let p = plan(temp.path());
    let (notes, log) = run(script(), &p, Vec::new());
    assert_eq!(notes.last(), Some(&end(60_000, RecordEnd::Longest)));
    assert_eq!(times(&notes).len(), 60);
    assert_eq!(log.frames.len(), 60 * usize::from(FPS));
    assert!(log.finished && p.file.exists());
}

#[test]
fn a_run_that_falls_behind_still_tells_each_second_and_stops_at_the_longest_length() {
    let temp = Temp::new();
    let mut s = script();
    s.frame_cost = slow();
    let (notes, log) = run(s, &plan(temp.path()), Vec::new());
    assert_eq!(notes.last(), Some(&end(60_000, RecordEnd::Longest)));
    let each: Vec<u32> = (0..60).map(|s| s * 1000).collect();
    assert_eq!(times(&notes), each);
    let cap = 60 * u64::from(FPS);
    assert!(log.frames.iter().all(|(n, _)| *n < cap));
    assert!(log.frames.len() < usize::try_from(cap).unwrap());
}

#[test]
fn the_floor_reached_mid_run_ends_it_and_keeps_the_video() {
    let temp = Temp::new();
    let p = plan(temp.path());
    let mut s = script();
    s.room.push((secs(2.0), Some(LOW)));
    let (notes, _) = run(s, &p, Vec::new());
    assert_eq!(times(&notes), [0, 1000]);
    assert_eq!(notes.last(), Some(&end(2000, RecordEnd::Full)));
    assert!(p.file.exists());
}

#[test]
fn a_switch_is_acked_and_a_refused_one_answers_its_fault_while_the_run_goes_on() {
    let temp = Temp::new();
    let (mut s, mut p) = (script(), plan(temp.path()));
    s.mic = Voice::refused(RecordFault::MicRefused);
    p.mic = false;
    let orders = vec![
        (secs(0.5), Order::Switch(Switch::Mic, true)),
        (secs(0.7), Order::Switch(Switch::Sound, false)),
        (secs(0.8), Order::Switch(Switch::Sound, false)),
        (secs(1.5), Order::Stop),
    ];
    let (notes, _) = run(s, &p, orders);
    let off = RecordNote::Switched {
        id: ID,
        switch: Switch::Sound,
        on: false,
    };
    let want = [
        RecordNote::Time { id: ID, ms: 0 },
        fault(RecordFault::MicRefused),
        off.clone(),
        off,
        RecordNote::Time { id: ID, ms: 1000 },
        end(1500, RecordEnd::Stopped),
    ];
    assert_eq!(notes, want);
    assert!(p.file.exists());
}

#[test]
fn a_broken_file_stops_the_run_and_is_deleted() {
    let temp = Temp::new();
    let p = plan(temp.path());
    let mut s = script();
    s.sink_fails_at = Some(40);
    let (notes, _) = run(s, &p, Vec::new());
    assert_eq!(times(&notes), [0, 1000]);
    assert_eq!(notes.last(), Some(&fault(RecordFault::Failed)));
    assert!(!p.file.exists());
}

#[test]
fn a_stop_before_the_first_frame_still_gives_one() {
    let temp = Temp::new();
    let p = plan(temp.path());
    let (notes, log) = run(script(), &p, vec![(Duration::ZERO, Order::Stop)]);
    assert_eq!(notes.last(), Some(&end(33, RecordEnd::Stopped)));
    assert_eq!(log.frames.len(), 1);
}

#[test]
fn the_part_is_made_even_as_h264_wants() {
    let temp = Temp::new();
    let shape = plan(temp.path()).shape();
    assert_eq!((shape.width, shape.height, shape.fps), (640, 360, 30));
}
