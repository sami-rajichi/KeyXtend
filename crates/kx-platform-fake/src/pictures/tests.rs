//! The fake pictures, held to the port: the session's config recorded, scripted answers in turn, failures that stand
//! for a refused, hung or dead worker, and counted asks; nothing sleeps.

use super::*;
use kx_module_api::Redacted;
use kx_module_api::ports::{ImageCodec, RGBA_BYTES};
use std::thread;

/// How long a test waits for another thread; it never sleeps.
const WITHIN: Duration = Duration::from_secs(10);

/// Test values; the real ones come from the clipboard module's settings.
const CFG: PictureConfig = PictureConfig {
    deadline_ms: 2_000,
    idle_s: 30,
    memory_mb: 256,
    max_pixels: 40_000_000,
};

/// The thumbnail side the tests ask for.
const SIDE: u32 = 4;

/// One opaque red pixel.
const RED: [u8; RGBA_BYTES] = [255, 0, 0, 255];

/// `w` by `h` red pixels, drawn within `side`.
fn red(w: u32, h: u32, side: u32) -> Pixels {
    let n = usize::try_from(w * h).unwrap();
    Pixels::new(w, h, RED.repeat(n), side).unwrap()
}

/// An ask for a PNG of `bytes` within `side`.
fn ask(bytes: &[u8], side: u32) -> PictureAsk {
    PictureAsk {
        codec: ImageCodec::Png,
        bytes: Redacted::new(bytes.to_vec()),
        side,
    }
}

#[test]
fn open_records_the_config_and_a_dropped_session_stops() {
    let pictures = FakePictures::new();
    assert_eq!((pictures.opens(), pictures.config()), (0, None));
    let run = pictures.open(CFG);
    assert!(pictures.running());
    assert_eq!((pictures.opens(), pictures.config()), (1, Some(CFG)));
    drop(run);
    assert!(!pictures.running());
    assert_eq!(pictures.config(), Some(CFG), "kept after the stop");
}

#[test]
fn scripted_answers_come_in_turn_and_the_latest_repeats() {
    let pictures = FakePictures::new();
    let mut run = pictures.open(CFG);
    let (wide, tall) = (red(SIDE, 2, SIDE), red(1, SIDE, SIDE));
    pictures.queue(Ok(wide.clone()));
    pictures.queue(Err(PictureError::Refused));
    pictures.queue(Ok(tall.clone()));
    assert_eq!(run.draw(&ask(b"a", SIDE)), Ok(wide));
    assert_eq!(run.draw(&ask(b"b", SIDE)), Err(PictureError::Refused));
    assert_eq!(run.draw(&ask(b"c", SIDE)), Ok(tall.clone()));
    assert_eq!(run.draw(&ask(b"d", SIDE)), Ok(tall), "the latest repeats");
}

#[test]
fn before_any_script_the_worker_is_gone() {
    let mut run = FakePictures::new().open(CFG);
    assert_eq!(run.draw(&ask(b"a", SIDE)), Err(PictureError::Gone));
}

#[test]
fn a_hang_is_a_scripted_timeout_that_never_sleeps_and_every_error_comes_back() {
    let pictures = FakePictures::new();
    let mut run = pictures.open(CFG);
    let errors = [
        PictureError::Timeout,
        PictureError::Gone,
        PictureError::TooBig,
        PictureError::Refused,
    ];
    for error in errors {
        pictures.queue(Err(error));
        assert_eq!(run.draw(&ask(b"a", SIDE)), Err(error));
    }
}

#[test]
fn pixels_past_the_asks_side_come_back_gone_as_bad_pixels_do() {
    let pictures = FakePictures::new();
    let mut run = pictures.open(CFG);
    pictures.queue(Ok(red(SIDE, SIDE, SIDE)));
    assert_eq!(run.draw(&ask(b"a", SIDE - 1)), Err(PictureError::Gone));
    assert!(run.draw(&ask(b"a", SIDE)).is_ok(), "within the side");
}

#[test]
fn asks_are_counted_and_the_latest_kept_failed_ones_too() {
    let pictures = FakePictures::new();
    let mut run = pictures.open(CFG);
    assert_eq!((pictures.asks(), pictures.last_ask()), (0, None));
    let _ = run.draw(&ask(b"first", SIDE));
    pictures.queue(Ok(red(1, 1, SIDE)));
    let _ = run.draw(&ask(b"second", SIDE));
    assert_eq!(pictures.asks(), 2);
    assert_eq!(pictures.last_ask(), Some(ask(b"second", SIDE)));
}

#[test]
fn a_draw_on_another_thread_wakes_a_waiting_test() {
    let pictures = FakePictures::new();
    assert!(!pictures.wait_asks(1, Duration::ZERO), "no ask yet");
    let port: Arc<dyn Pictures> = Arc::new(pictures.clone());
    let media = thread::spawn(move || port.open(CFG).draw(&ask(b"a", SIDE)));
    assert!(pictures.wait_asks(1, WITHIN));
    assert_eq!(media.join().unwrap(), Err(PictureError::Gone));
}

#[test]
fn debug_never_prints_the_bytes() {
    let pictures = FakePictures::new();
    let _ = pictures.open(CFG).draw(&ask(b"secret", SIDE));
    let text = format!("{pictures:?}");
    assert!(text.contains("FakePictures"));
    assert!(!text.contains("115"), "no byte of the ask");
}
