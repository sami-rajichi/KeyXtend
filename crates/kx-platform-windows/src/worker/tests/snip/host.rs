//! The recorder host over the real worker, live: a run of a part our window fills, with no sound, tells its time in
//! order and stops into a kept video of the unsaved part; a 2× copy is made beside it; a cancelled run leaves no file;
//! and the Snips folder adapter lists and deletes what is left. The facts read Media Foundation as there.

use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};

use kx_module_api::ports::{
    FolderPath, MicSeal, Part, RecordAsk, RecordEnd, RecordNote, RecordWaits, Recorder, Rect,
    SnipFile, SnipFolder, Speed, SpeedAsk,
};
use kx_platform::snip::{DiskSnips, WorkerRecorder};
use kx_platform::{SNIP_DIR, UNSAVED_DIR};
use windows::Win32::Foundation::RECT;

use super::screen::{Plain, Rgb, inset, physical, until, work_area};
use super::session::{FPS, MS_S, RECORD_MB};
use super::{Temp, WAIT, hold, spawner};
use crate::{QpcClock, WinDisk, WinKit};

/// The part's size, even as H.264 wants.
const SIDE: (i32, i32) = (320, 240);
/// The window filling the part.
const BACK: Rgb = Rgb(200, 120, 0);
/// How long the kept run records, and the cancelled one before its cancel.
const SECONDS: u32 = 2;
/// Each job wait at its longest, as a real PC may be slow.
const WAITS: RecordWaits = RecordWaits {
    start_ms: *RecordWaits::START_MS.end(),
    tick_ms: *RecordWaits::TICK_MS.end(),
    end_ms: *RecordWaits::END_MS.end(),
    switch_ms: *RecordWaits::SWITCH_MS.end(),
    speed_ms: *RecordWaits::SPEED_MS.end(),
    speed_mb_ms: *RecordWaits::SPEED_MB_MS.end(),
};

#[test]
#[cfg_attr(not(kx_live_snip), ignore = "records the screen; opt-in")]
fn the_recorder_host_keeps_a_stopped_run_copies_it_and_leaves_nothing_of_a_cancelled_one() {
    let (_live, _px, temp) = (hold(), physical(), Temp::new());
    let at = inset(work_area(), 0, SIDE.0, SIDE.1);
    let back = Plain::new(at, BACK);
    back.show();
    let snips = folder(&temp);
    let recorder = recorder(&snips);
    assert!(recorder.facts().video, "this PC has Media Foundation");
    let ask = RecordAsk {
        part: Part::new(rect(at)).unwrap(),
        fps: FPS,
        mic: None,
        sound: false,
        max_min: *RecordAsk::MAX_MIN.start(),
        min_free_mb: *RecordAsk::MIN_FREE_MB.start(),
        job_mb: RECORD_MB,
        waits: WAITS,
    };
    let kept = stopped(&*recorder, ask.clone());
    let copy = recorder.speed(&speed(kept.clone())).unwrap();
    assert!(size(&copy) > 0 && copy != kept, "a copy of its own");
    cancelled(&*recorder, ask);
    let left = snips.unsaved().unwrap();
    assert_eq!(left.len(), 2, "the kept video and its copy");
    for file in left {
        snips.remove(&file.file).unwrap();
    }
    assert!(snips.unsaved().unwrap().is_empty());
}

/// A run of `ask` stopped after `SECONDS`, its times in order and its video kept.
fn stopped(recorder: &dyn Recorder, ask: RecordAsk) -> SnipFile {
    let (tell, heard) = ear();
    let mut run = recorder.start(ask, tell).unwrap();
    let times = times_until(&heard, SECONDS * MS_S);
    assert!(
        times.windows(2).all(|w| w[0] <= w[1]),
        "in order: {times:?}"
    );
    run.stop().unwrap();
    let mut done = None;
    assert!(until(WAIT, || {
        done = done.or_else(|| match heard.try_recv() {
            Ok(RecordNote::Done(done)) => Some(done),
            _ => None,
        });
        done.is_some()
    }));
    let done = done.unwrap();
    eprintln!(
        "host: kept {} ms, {} bytes",
        done.length_ms,
        size(run.file())
    );
    assert_eq!(done.end, RecordEnd::Stopped);
    assert!(done.length_ms >= SECONDS * MS_S && size(run.file()) > 0);
    run.file().clone()
}

/// A run of `ask` cancelled after a second leaves no file.
fn cancelled(recorder: &dyn Recorder, ask: RecordAsk) {
    let (tell, heard) = ear();
    let mut run = recorder.start(ask, tell).unwrap();
    times_until(&heard, MS_S);
    let file = run.file().clone();
    run.cancel();
    drop(run);
    assert!(
        until(WAIT, || !file.expose().exists()),
        "a cancel leaves no file"
    );
}

/// The times told until one reaches `ms`.
fn times_until(heard: &Receiver<RecordNote>, ms: u32) -> Vec<u32> {
    let mut times = Vec::new();
    let reached = until(WAIT, || {
        times.extend(heard.try_iter().filter_map(|note| match note {
            RecordNote::Time { ms } => Some(ms),
            _ => None,
        }));
        times.last().is_some_and(|&last| last >= ms)
    });
    assert!(reached, "the run told its time: {times:?}");
    times
}

/// The Snips folder in `temp`, with its unsaved part made.
fn folder(temp: &Temp) -> Arc<DiskSnips> {
    let snips = temp.path().join(SNIP_DIR);
    std::fs::create_dir_all(snips.join(UNSAVED_DIR)).unwrap();
    let snips = FolderPath::new(snips).unwrap();
    Arc::new(DiskSnips::new(Some(snips), Arc::new(WinDisk)))
}

/// The recorder over the real worker, into `snips`.
fn recorder(snips: &Arc<DiskSnips>) -> Arc<dyn Recorder> {
    let clock = Arc::new(QpcClock::new().unwrap());
    let (seal, kit) = (MicSeal::new(), Arc::new(WinKit));
    let folder: Arc<dyn SnipFolder> = snips.clone();
    Arc::new(WorkerRecorder::new(spawner(), clock, seal, folder, kit))
}

/// A 2× copy of `from`.
fn speed(from: SnipFile) -> SpeedAsk {
    SpeedAsk {
        from,
        speed: Speed::new(2).unwrap(),
        min_free_mb: *RecordAsk::MIN_FREE_MB.start(),
        job_mb: RECORD_MB,
        waits: WAITS,
    }
}

/// A callback handing each note on, and where they arrive.
fn ear() -> (Box<dyn Fn(RecordNote) + Send>, Receiver<RecordNote>) {
    let (said, heard) = mpsc::channel();
    let tell = move |note| {
        let _ = said.send(note);
    };
    (Box::new(tell), heard)
}

/// `file`'s size in bytes, 0 when it is not there.
fn size(file: &SnipFile) -> u64 {
    std::fs::metadata(file.expose()).map_or(0, |m| m.len())
}

/// The port's rectangle for a window's box.
fn rect(at: RECT) -> Rect {
    let (w, h) = (at.right - at.left, at.bottom - at.top);
    Rect {
        x: at.left,
        y: at.top,
        w,
        h,
    }
}
