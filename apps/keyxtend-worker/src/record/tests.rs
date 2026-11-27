//! The record job over a scripted studio: a run told in order, its switches, Stop and Cancel, one piece of work at a
//! time, its ends and faults, and re-timed copies. None sleeps: the studio's clock moves only when a test lets it, and
//! files live in a folder under the target folder.

mod copy;
mod ends;
mod host;

use host::Host;
use kx_ipc::{Area, MS_PER_S, RecordAsk, RecordEnd, RecordFault, RecordNote, SNIP_FPS};
use kx_ipc::{SNIP_MIN_FREE_MB, Switch};
use kx_platform_record::fake::{FakeStudio, Script, Tape, Voice};
use kx_platform_record::record::{RATE, Shape};
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

/// The longest the test waits for the job's next write before it fails; no test sleeps.
const WAIT: Duration = Duration::from_secs(20);
/// The run most tests start.
const ID: u32 = 7;
/// An id no run has: a switch on it is answered at once, so the test knows every ask before it was taken.
const NONE: u32 = 99;
/// The run's rate, the higher of `SNIP_FPS`, and the length of its first frame in ms.
const FPS: u8 = SNIP_FPS[1];
const ONE_FRAME_MS: u32 = MS_PER_S / FPS as u32;
/// The floor every run keeps, the bottom of `SNIP_MIN_FREE_MB`.
const FLOOR: u32 = *SNIP_MIN_FREE_MB.start();
/// What the fake file holds once ended.
const ENDED: &[u8] = b"ended";
/// The start of each test's folder name in the test program's profile folder.
const ROOT: &str = "kx-record-test";

/// Makes each test's folder name unique within this process.
static UNIQUE: AtomicU32 = AtomicU32::new(0);

/// A fresh folder under the target folder, deleted when dropped, even when the test fails.
struct Dir(PathBuf);

impl Dir {
    fn new() -> Self {
        let exe = std::env::current_exe().unwrap();
        let profile = exe.parent().and_then(Path::parent).unwrap();
        let n = UNIQUE.fetch_add(1, Ordering::Relaxed);
        let path = profile.join(format!("{ROOT}-{}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    /// A video's path in the folder.
    fn file(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A working script with a one-second tape.
fn script() -> Script {
    Script::working(Tape {
        shape: Shape {
            width: 640,
            height: 360,
            fps: u32::from(FPS),
        },
        frames: u64::from(FPS),
        step: 1,
        sound: u64::from(RATE),
        level: 1_000,
        length_ms: MS_PER_S,
    })
}

/// A studio playing `s` whose clock waits for the test's passes.
fn held(s: Script) -> FakeStudio {
    let (studio, _) = FakeStudio::new(s, Vec::new());
    studio.hold();
    studio
}

/// Run `ID`, as `start_as` makes it.
fn start(file: &Path) -> RecordAsk<'static> {
    start_as(ID, file)
}

/// A one-minute run `id` of a part into `file`, with the microphone on and the PC's sound off.
fn start_as(id: u32, file: &Path) -> RecordAsk<'static> {
    RecordAsk::Start {
        id,
        area: Area {
            x: 0,
            y: 0,
            w: 640,
            h: 360,
        },
        fps: FPS,
        mic: true,
        sound: false,
        max_min: 1,
        min_free_mb: FLOOR,
        file: Cow::Owned(file.to_string_lossy().into_owned()),
    }
}

/// The time note of run `ID`.
fn time(ms: u32) -> RecordNote {
    RecordNote::Time { id: ID, ms }
}

/// The end note of run `ID`.
fn end(length_ms: u32, end: RecordEnd) -> RecordNote {
    RecordNote::End {
        id: ID,
        length_ms,
        end,
    }
}

#[test]
fn a_run_tells_its_start_then_its_end_on_stop_and_keeps_the_video() {
    let dir = Dir::new();
    let file = dir.file("snip 1.mp4");
    let studio = held(script());
    let mut host = Host::start(&studio);
    host.send(&start(&file));
    studio.grant(1);
    assert_eq!(host.next(), time(0));
    host.send(&RecordAsk::Stop { id: ID });
    assert_eq!(host.barrier(), []);
    studio.free();
    assert_eq!(host.next(), end(ONE_FRAME_MS, RecordEnd::Stopped));
    assert_eq!(fs::read(&file).unwrap(), ENDED);
    assert_eq!(host.finish(true), (Ok(()), Vec::new()));
}

#[test]
fn cancel_deletes_the_video_and_tells_nothing_more() {
    let dir = Dir::new();
    let file = dir.file("snip 1.mp4");
    let studio = held(script());
    let mut host = Host::start(&studio);
    host.send(&start(&file));
    studio.grant(1);
    assert_eq!(host.next(), time(0));
    host.send(&RecordAsk::Cancel { id: ID });
    assert_eq!(host.barrier(), []);
    studio.free();
    assert_eq!(host.finish(true), (Ok(()), Vec::new()));
    assert!(!file.exists());
}

#[test]
fn each_switch_reaches_the_run_and_is_answered_once_made() {
    let dir = Dir::new();
    let studio = held(script());
    let mut host = Host::start(&studio);
    host.send(&start(&dir.file("snip 1.mp4")));
    studio.grant(1);
    assert_eq!(host.next(), time(0));
    host.send(&RecordAsk::Mic { id: ID, on: false });
    host.send(&RecordAsk::Sound { id: ID, on: true });
    host.send(&RecordAsk::Stop { id: ID });
    assert_eq!(host.barrier(), []);
    studio.free();
    let switched = |switch, on| RecordNote::Switched { id: ID, switch, on };
    assert_eq!(host.next(), switched(Switch::Mic, false));
    assert_eq!(host.next(), switched(Switch::Sound, true));
    assert_eq!(host.next(), end(ONE_FRAME_MS, RecordEnd::Stopped));
}

#[test]
fn a_refused_switch_answers_its_fault_and_the_run_goes_on() {
    let dir = Dir::new();
    let mut s = script();
    s.sound = Voice::refused(RecordFault::NoSound);
    let studio = held(s);
    let mut host = Host::start(&studio);
    host.send(&start(&dir.file("snip 1.mp4")));
    studio.grant(1);
    assert_eq!(host.next(), time(0));
    host.send(&RecordAsk::Sound { id: ID, on: true });
    host.send(&RecordAsk::Stop { id: ID });
    assert_eq!(host.barrier(), []);
    studio.free();
    let fault = RecordFault::NoSound;
    assert_eq!(host.next(), RecordNote::Fault { id: ID, fault });
    assert_eq!(host.next(), end(ONE_FRAME_MS, RecordEnd::Stopped));
}

#[test]
fn one_piece_of_work_at_a_time_and_asks_for_no_run_are_answered_or_dropped() {
    let dir = Dir::new();
    let studio = held(script());
    let mut host = Host::start(&studio);
    host.send(&start(&dir.file("snip 1.mp4")));
    studio.grant(1);
    assert_eq!(host.next(), time(0));
    host.send(&start_as(ID + 1, &dir.file("snip 2.mp4")));
    host.send(&copy::ask(
        ID + 2,
        &dir.file("snip 1.mp4"),
        2,
        &dir.file("fast.mp4"),
    ));
    host.send(&RecordAsk::Stop { id: NONE });
    host.send(&RecordAsk::Cancel { id: NONE });
    let busy = |id| RecordNote::Fault {
        id,
        fault: RecordFault::Failed,
    };
    assert_eq!(host.barrier(), [busy(ID + 1), busy(ID + 2)]);
    assert!(!dir.file("snip 2.mp4").exists() && !dir.file("fast.mp4").exists());
    host.send(&RecordAsk::Stop { id: ID });
    assert_eq!(host.barrier(), []);
    studio.free();
    assert_eq!(host.next(), end(ONE_FRAME_MS, RecordEnd::Stopped));
}
