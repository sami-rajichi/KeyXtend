//! The record job, live: a worker under its job and restricted token records the work area with sound into a playable
//! MP4 at 30 fps, here and in weak mode, with the pointer and without the border as its capture reads them back (else
//! it refuses the start), and a shielded window stays out until unshielded. Here it also re-times that video at 2× and
//! a very short run at 4× (edge 20), and cancels a run, which leaves no file; our windows fill the part.

use std::path::{Path, PathBuf};

use kx_ipc::{Area, CLEAN_EXIT, RecordAsk, RecordNote};
use kx_platform_record::record::RATE;
use windows::Win32::Foundation::RECT;
use windows::Win32::Media::MediaFoundation::{MFAudioFormat_AAC, MFVideoFormat_H264};

use super::screen::{
    Plain, Rgb, area, centred, count, frame, grab, inset, physical, until, work_area,
};
use super::session::{FPS, MS_S, Session};
use super::{Temp, WAIT, hold, mp4, spawner, weak};

/// The shielded window's size, centred on the part, so the check holds whichever way a decoded frame's rows run.
const SHIELDED: (i32, i32) = (640, 360);
/// A small window at the part's corner, shown and hidden each frame, so the screen changes as a busy one does.
const TICK: (i32, i32, i32) = (8, 64, 64);
/// The backdrop's, the shielded window's and the ticker's colours.
const COLOURS: [Rgb; 3] = [Rgb(200, 0, 200), Rgb(0, 200, 0), Rgb(0, 0, 200)];
/// The length recorded.
const SECONDS: u32 = 3;
/// How far a file's length may stray from the length told: the sound encoder's own start.
const SLACK_MS: u32 = 200;
/// How much later than the clock the run may tell its last time: a run that cannot keep up tells it later.
const LAG_MS: u128 = 200;
/// The share of the centre a decoded frame must show as the backdrop, in percent; the pointer may cover a little.
const BACK_PCT: usize = 95;
/// The first run's id; the others count on from it.
const ID: u32 = 1;
/// The files' names in the test's folder.
const FILE: &str = "snip.mp4";
const FAST: &str = "snip 2x.mp4";
const SHORT: &str = "short.mp4";
const QUICK: &str = "short 4x.mp4";
const CANCELLED: &str = "cancelled.mp4";

#[test]
#[cfg_attr(
    not(kx_live_snip),
    ignore = "records the screen, the microphone and the PC's sound; opt-in"
)]
fn the_record_job_makes_a_playable_mp4_with_sound_and_re_times_it() {
    let (_live, _px, temp) = (hold(), physical(), Temp::new());
    let scene = Scene::show();
    let mut session = Session::start(&spawner());
    let video = recorded(&mut session, &scene, temp.path(), "normal");
    re_times(&mut session, &scene, &video, temp.path());
    let cancelled = cancels(&mut session, &scene, temp.path());
    ended(session, "normal");
    assert!(!cancelled.exists(), "a cancelled run leaves no file");
    scene.unshielded_shows("normal");
}

#[test]
#[cfg_attr(
    not(kx_live_snip),
    ignore = "records the screen, the microphone and the PC's sound; opt-in"
)]
fn a_weak_worker_still_records_at_thirty_fps() {
    let (_live, _px, temp) = (hold(), physical(), Temp::new());
    let scene = Scene::show();
    let mut session = Session::start(&spawner().capped(weak()));
    recorded(&mut session, &scene, temp.path(), "weak");
    ended(session, "weak");
    scene.unshielded_shows("weak");
}

/// Our windows for a recording: the backdrop over the part, the shielded window on it and the ticker.
struct Scene {
    part: Area,
    _back: Plain,
    shielded: Plain,
    shielded_at: RECT,
    tick: Plain,
}

impl Scene {
    /// The part is the whole work area, at even sizes as H.264 wants: what Video full screen records, less the taskbar.
    fn show() -> Self {
        let work = work_area();
        let even = |n: i32| n - n % 2;
        let (w, h) = (even(work.right - work.left), even(work.bottom - work.top));
        let at = inset(work, 0, w, h);
        let [back, front, tick] = COLOURS;
        let back = Plain::new(at, back);
        back.show();
        let shielded_at = centred(at, SHIELDED.0, SHIELDED.1);
        let shielded = Plain::new(shielded_at, front);
        shielded.shield(true);
        shielded.show();
        let (gap, tw, th) = TICK;
        let tick = Plain::new(inset(at, gap, tw, th), tick);
        assert!(
            until(WAIT, || count(&grab(at), COLOURS[0]) * 2 > area(at)),
            "the backdrop shows"
        );
        let part = Area {
            x: at.left,
            y: at.top,
            w,
            h,
        };
        Self {
            part,
            _back: back,
            shielded,
            shielded_at,
            tick,
        }
    }

    /// The part's size, as the video's frames hold it.
    fn size(&self) -> (u32, u32) {
        let side = |n: i32| u32::try_from(n).unwrap();
        (side(self.part.w), side(self.part.h))
    }

    /// Shows or hides the ticker by turns, a frame at a time, so the screen keeps changing.
    fn ticker(&self) -> impl FnMut() + '_ {
        let mut shown = false;
        move || {
            shown = !shown;
            if shown {
                self.tick.show();
            } else {
                self.tick.hide();
            }
            frame();
        }
    }

    /// Unshields the shielded window and sees it in a grab, so its absence from the video means something.
    fn unshielded_shows(&self, mode: &str) {
        let at = self.shielded_at;
        self.shielded.shield(false);
        assert!(
            until(WAIT, || count(&grab(at), COLOURS[1]) == area(at)),
            "{mode}: unshielded, the window shows, so it was there all along"
        );
    }
}

/// Records `SECONDS` into the folder `dir` and checks the run's notes and its file; `mode` names the run.
fn recorded(session: &mut Session, scene: &Scene, dir: &Path, mode: &str) -> PathBuf {
    let video = dir.join(FILE);
    let run = session.record(ID, (scene.part, &video), SECONDS, &mut scene.ticker());
    eprintln!("record {mode}: {run:?}");
    let want: Vec<u32> = (0..=SECONDS).map(|s| s * MS_S).collect();
    // The time at 0 says the capture started, which it does only with the pointer on and the border off.
    assert_eq!(run.times, want, "{mode}: a time each second");
    let late = run
        .wall
        .as_millis()
        .saturating_sub(u128::from(SECONDS * MS_S));
    assert!(
        late <= LAG_MS,
        "{mode}: the run told its last time {late} ms late"
    );
    playable(&video, scene.size(), run.length_ms, mode);
    heard(&video, run.length_ms, mode);
    let (found, total) = mp4::centre(&video, frames(run.length_ms) / 2, &COLOURS[..2]);
    eprintln!(
        "record {mode}: centre of a middle frame, near backdrop / shielded of {total}: {found:?}"
    );
    assert_eq!(found[1], 0, "{mode}: the shielded window is in the video");
    assert!(
        found[0] * 100 >= total * BACK_PCT,
        "{mode}: the backdrop shows through"
    );
    video
}

/// Copies `video` at 2×, then records a very short run and copies it at 4× (edge 20); each copy plays.
fn re_times(session: &mut Session, scene: &Scene, video: &Path, dir: &Path) {
    let source = mp4::read(video).length_ms;
    let fast = dir.join(FAST);
    let length = session.speed(ID + 1, video, 2, &fast);
    eprintln!("record 2x: {length} ms of {source} ms");
    playable(&fast, scene.size(), length, "2x");
    let short = dir.join(SHORT);
    let run = session.record(ID + 2, (scene.part, &short), 0, &mut scene.ticker());
    let quick = dir.join(QUICK);
    let length = session.speed(ID + 3, &short, 4, &quick);
    eprintln!(
        "record 4x of a short run: {length} ms of {} ms",
        run.length_ms
    );
    assert!(length > 0, "a copy keeps a frame at least");
    playable(&quick, scene.size(), length, "4x short");
}

/// Starts a run and cancels it once it tells its start; hands back its file, which must be gone once the job ended.
fn cancels(session: &mut Session, scene: &Scene, dir: &Path) -> PathBuf {
    let file = dir.join(CANCELLED);
    let id = ID + 4;
    session.start_run(id, scene.part, &file);
    let note = session.next(&mut scene.ticker());
    assert_eq!(note, RecordNote::Time { id, ms: 0 });
    session.send(&RecordAsk::Cancel { id });
    file
}

/// Says goodbye and checks the worker ended cleanly; `mode` names the run.
fn ended(session: Session, mode: &str) {
    let ran = session.bye();
    eprintln!("record {mode}: peak {:?} MB", ran.peak_mb);
    assert_eq!(ran.code, Some(u32::from(CLEAN_EXIT)), "{mode}");
}

/// The frames a video of `length_ms` holds at `FPS`.
fn frames(length_ms: u32) -> u32 {
    (length_ms * u32::from(FPS)).div_ceil(MS_S)
}

/// Checks `path` reads back as an H.264 video of `size` at `FPS` holding `length_ms`, with an AAC track.
fn playable(path: &Path, size: (u32, u32), length_ms: u32, mode: &str) {
    let facts = mp4::read(path);
    eprintln!("record {mode}: {facts:?}");
    assert!(
        facts.length_ms.abs_diff(u64::from(length_ms)) <= u64::from(SLACK_MS),
        "{mode}: told {length_ms} ms"
    );
    assert_eq!(facts.size, size, "{mode}");
    assert_eq!(facts.video, MFVideoFormat_H264, "{mode}");
    assert_eq!(facts.audio, MFAudioFormat_AAC, "{mode}");
    assert_eq!(facts.rate, (u32::from(FPS), 1), "{mode}");
    assert_eq!(facts.frames, frames(length_ms), "{mode}");
}

/// Checks the sound track covers the video, and says how much of it is not silent; never the sound itself.
fn heard(path: &Path, length_ms: u32, mode: &str) {
    let track = mp4::sound(path);
    let per_ms = u64::from(RATE / MS_S);
    let (frames, loud) = (track.frames, track.loud_pct);
    eprintln!("record {mode}: {frames} sound frames, {loud}% of samples not silent");
    let want = u64::from(length_ms) * per_ms;
    assert!(
        track.frames.abs_diff(want) <= u64::from(SLACK_MS) * per_ms,
        "{mode}: {} sound frames for {length_ms} ms",
        track.frames
    );
}
