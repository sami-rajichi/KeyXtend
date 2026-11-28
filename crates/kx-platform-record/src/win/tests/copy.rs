//! Media Foundation's writer and reader with no screen and no sound device: a video reads back frame by frame, its 2×
//! copy keeps every second frame, a frame never reads past its buffer and a file left unended can be deleted at once.
//! Without Media Foundation's libraries or encoder each test is skipped, as the job answers `NoVideo` there.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use windows::Win32::Media::MediaFoundation::{
    MFCreate2DMediaBuffer, MFCreateSample, MFVideoFormat_NV12,
};

use super::super::cut::{Cut, picture};
use super::super::reel::{Decoded, WinReel};
use super::super::studio::WinStudio;
use super::super::writer::memory;
use crate::record::tests::Temp;
use crate::record::{CHANNELS, Piece, RATE, RecordFault, Reel, Shape, Sink, Studio, scrap};
use crate::speed::speed;
use crate::table::table;

/// The video written: a small part at 30 fps, one second long.
const SHAPE: Shape = Shape {
    width: 320,
    height: 240,
    fps: 30,
};
const FRAMES: u64 = 30;
/// NV12 holds three bytes for every two pixels, and grey is the middle of each byte.
const NV12: (usize, usize) = (3, 2);
const GREY: u8 = 128;
/// The sound's steady sample.
const LEVEL: i16 = 1_000;
/// How far the sound read back may stray from the sound written: the AAC encoder's own start and end, in frames.
const SLACK: u64 = 4_096;
/// The longest the reading may take before the test fails instead of hanging.
const WAIT: Duration = Duration::from_secs(60);

/// A grey frame of `SHAPE` in memory.
#[allow(unsafe_code, reason = "Makes a sample of ours.")]
fn grey() -> Decoded {
    let side = |n: u32| usize::try_from(n).unwrap();
    let (w, h) = (side(SHAPE.width), side(SHAPE.height));
    let buffer = memory(&vec![GREY; w * h * NV12.0 / NV12.1]).unwrap();
    // SAFETY: a new sample holding a live buffer.
    let sample = unsafe {
        let sample = MFCreateSample().unwrap();
        sample.AddBuffer(&buffer).unwrap();
        sample
    };
    Decoded {
        sample,
        size: (SHAPE.width, SHAPE.height),
    }
}

/// A decoder's kind of frame: a 2D NV12 buffer of `SHAPE` in a sample, said to hold `size`.
#[allow(unsafe_code, reason = "Makes a sample of ours.")]
fn flat(size: (u32, u32)) -> Decoded {
    let nv12 = MFVideoFormat_NV12.data1;
    // SAFETY: a new 2D buffer of plain sizes, in a new sample.
    let sample = unsafe {
        let buffer = MFCreate2DMediaBuffer(SHAPE.width, SHAPE.height, nv12, false).unwrap();
        let sample = MFCreateSample().unwrap();
        sample.AddBuffer(&buffer).unwrap();
        sample
    };
    Decoded { sample, size }
}

/// The bytes of an NV12 picture of `shape`.
fn bytes(shape: Shape) -> usize {
    let side = |n: u32| usize::try_from(n).unwrap();
    side(shape.width) * side(shape.height) * NV12.0 / NV12.1
}

/// The sound frames of one video frame.
fn per_frame() -> u64 {
    u64::from(RATE) / u64::from(SHAPE.fps)
}

/// Writes `FRAMES` grey frames with steady sound into `path`; `Err(NoVideo)` without Media Foundation's encoder.
fn made(path: &Path) -> Result<(), RecordFault> {
    let mut cut = Cut::open(path, SHAPE, table().unwrap())?;
    let frame = grey();
    let pcm = vec![LEVEL; usize::try_from(per_frame()).unwrap() * usize::from(CHANNELS)];
    for n in 0..FRAMES {
        cut.frame(n, &frame)?;
        cut.sound(n * per_frame(), &pcm)?;
    }
    Box::new(cut).finish()
}

/// Writes frames into `path`, drops the writer unended and deletes the file while Media Foundation still runs; whether
/// it went, or `Err(NoVideo)` without Media Foundation's encoder.
fn left(path: &Path) -> Result<bool, RecordFault> {
    let mut cut = Cut::open(path, SHAPE, table().unwrap())?;
    let frame = grey();
    for n in 0..FRAMES {
        cut.frame(n, &frame)?;
    }
    drop(cut);
    Ok(scrap(path))
}

/// Runs `work` with Media Foundation started, on a thread of its own, so a reader that never ends fails the test.
fn within<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ready = WinStudio.ready().unwrap();
        let _ = tx.send(work());
    });
    rx.recv_timeout(WAIT).expect("the work ends")
}

/// The frames and sound frames `path` reads back as.
fn counted(path: PathBuf) -> (u64, u64) {
    within(move || {
        let mut reel = WinReel::open(&path).unwrap();
        let (mut frames, mut sound) = (0, 0);
        while let Some(piece) = reel.next().unwrap() {
            match piece {
                Piece::Frame(..) => frames += 1,
                Piece::Sound(pcm) => {
                    sound += u64::try_from(pcm.len()).unwrap() / u64::from(CHANNELS);
                }
            }
        }
        (frames, sound)
    })
}

#[test]
fn a_video_reads_back_frame_by_frame_and_its_2x_copy_keeps_every_second_frame() {
    if WinStudio.ready().is_err() {
        return;
    }
    let temp = Temp::new();
    let (from, to) = (temp.path().join("a.mp4"), temp.path().join("a 2x.mp4"));
    let wrote = within({
        let from = from.clone();
        move || made(&from)
    });
    if wrote == Err(RecordFault::NoVideo) {
        return;
    }
    assert_eq!(wrote, Ok(()));
    let (frames, sound) = counted(from.clone());
    assert_eq!(frames, FRAMES);
    assert!(sound.abs_diff(FRAMES * per_frame()) <= SLACK, "{sound}");
    let copied = within({
        let (from, to) = (from.clone(), to.clone());
        move || speed(&WinStudio, &from, &to, 2)
    });
    assert_eq!(copied, Ok(500));
    let (frames, sound) = counted(to);
    assert_eq!(frames, FRAMES / 2);
    assert!(sound.abs_diff(FRAMES / 2 * per_frame()) <= SLACK, "{sound}");
}

#[test]
fn a_frame_that_claims_more_rows_than_its_buffer_holds_reads_only_its_buffer() {
    if WinStudio.ready().is_err() {
        return;
    }
    let tall = Shape {
        height: SHAPE.height * 10,
        ..SHAPE
    };
    let (whole, short) = within(move || {
        let fits = flat((SHAPE.width, SHAPE.height));
        let claims = flat((tall.width, tall.height));
        let read = |frame: &Decoded, shape| picture(frame, shape).unwrap().len();
        (read(&fits, SHAPE), read(&claims, tall))
    });
    assert_eq!(whole, bytes(SHAPE));
    assert!(short <= bytes(SHAPE), "{short} bytes read");
}

#[test]
fn a_file_left_unended_is_let_go_as_soon_as_its_writer_is_dropped() {
    if WinStudio.ready().is_err() {
        return;
    }
    let temp = Temp::new();
    let path = temp.path().join("left.mp4");
    let gone = within({
        let path = path.clone();
        move || left(&path)
    });
    if gone == Err(RecordFault::NoVideo) {
        return;
    }
    assert_eq!(gone, Ok(true));
    assert!(!path.exists());
}
