//! The loop, driven with in-memory streams: each draw gets its reply in order, a panic is a failure, and a broken
//! frame or stream ends the loop.

use super::*;
use crate::fixture::{Dib, rgba};
use crate::media;
use kx_ipc::{Ask, Reply};
use std::borrow::Cow;
use std::process::Command;
use std::{env, io};

/// The side the tests ask for.
const SIDE: u32 = 8;
/// The most pixels the tests allow.
const MAX_PIXELS: u64 = 1_000;
/// Bytes the test draw panics on.
const BOOM: &[u8] = b"boom";
/// Set only in the child process `a_hushed_panic_writes_nothing_on_stderr` starts.
const CHILD: &str = "KX_WORKER_HUSH_CHILD";
/// The child test that panics with the hook installed, by its full name.
const CHILD_TEST: &str = "serve::tests::hushed_child";
/// What the child writes on stdout once its panicking draw is answered.
const ANSWERED: &str = "hushed draw answered";

fn drawing(codec: Codec, bytes: &[u8], side: u32) -> Result<Picture, Why> {
    media::draw(codec, bytes, side, MAX_PIXELS)
}

fn draw_ask(id: u32, codec: Codec, bytes: &[u8]) -> Ask<'_> {
    Ask::Draw {
        id,
        codec,
        side: SIDE,
        bytes: Cow::Borrowed(bytes),
    }
}

fn frames(asks: &[Ask<'_>]) -> Vec<u8> {
    let mut out = Vec::new();
    for ask in asks {
        ask.write(&mut out).unwrap();
    }
    out
}

/// The tests' draw, which panics on `BOOM`.
fn test_draw(codec: Codec, bytes: &[u8], side: u32) -> Result<Picture, Why> {
    assert_ne!(bytes, BOOM, "the test draw panics");
    drawing(codec, bytes, side)
}

/// Every reply in `out`.
fn replies(out: &[u8]) -> Vec<Reply> {
    let mut written = out;
    std::iter::from_fn(|| Reply::read(&mut written).ok()).collect()
}

/// Runs the loop over `input`, returning its outcome and every reply it wrote.
fn run(input: &[u8]) -> (Result<(), IpcError>, Vec<Reply>) {
    let mut out = Vec::new();
    let outcome = serve(&mut &input[..], &mut out, test_draw);
    (outcome, replies(&out))
}

/// A 1 by 1 opaque DIB and its pixel.
fn tiny() -> (Vec<u8>, Vec<u8>) {
    (
        Dib::new(1, 1, 24, vec![3, 2, 1, 0]).bytes(),
        rgba(&[[1, 2, 3, 255]]),
    )
}

#[test]
fn a_draw_gets_its_pixels_and_bye_ends_the_loop() {
    let (dib, pixel) = tiny();
    let (outcome, replies) = run(&frames(&[draw_ask(7, Codec::Dib, &dib), Ask::Bye]));
    assert_eq!(outcome, Ok(()));
    let want = Reply::Pixels {
        id: 7,
        width: 1,
        height: 1,
        rgba: pixel,
    };
    assert_eq!(replies, [want]);
}

#[test]
fn a_bad_image_gets_a_failure_with_its_id() {
    let (outcome, replies) = run(&frames(&[draw_ask(9, Codec::Png, b"junk")]));
    assert_eq!(outcome, Ok(()));
    assert_eq!(
        replies,
        [Reply::Failed {
            id: 9,
            why: Why::Refused
        }]
    );
}

#[test]
fn too_many_pixels_get_too_big() {
    let big = Dib::new(100, 100, 24, Vec::new()).bytes();
    let (_, replies) = run(&frames(&[draw_ask(3, Codec::Dib, &big)]));
    assert_eq!(
        replies,
        [Reply::Failed {
            id: 3,
            why: Why::TooBig
        }]
    );
}

#[test]
fn a_panic_is_an_internal_failure_and_the_loop_goes_on() {
    let (dib, _) = tiny();
    let asks = [draw_ask(1, Codec::Dib, BOOM), draw_ask(2, Codec::Dib, &dib)];
    let (outcome, replies) = run(&frames(&asks));
    assert_eq!(outcome, Ok(()));
    assert_eq!(replies.len(), 2);
    assert_eq!(
        replies[0],
        Reply::Failed {
            id: 1,
            why: Why::Internal
        }
    );
    assert!(matches!(replies[1], Reply::Pixels { id: 2, .. }));
}

/// The panic hook is global, so the check runs in a child copy of this test program, filtered to `hushed_child`.
#[test]
fn a_hushed_panic_writes_nothing_on_stderr() {
    let out = Command::new(env::current_exe().unwrap())
        .args([CHILD_TEST, "--exact", "--nocapture", "--test-threads=1"])
        .env(CHILD, "1")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{stdout}");
    assert!(stdout.contains(ANSWERED), "{stdout}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.is_empty(), "{stderr}");
}

/// Does nothing unless it is that child: then it answers a panicking draw through the media job's own entry, so a
/// job that stops hushing fails the parent test.
#[test]
fn hushed_child() {
    if env::var_os(CHILD).is_none() {
        return;
    }
    let (input, mut out) = (frames(&[draw_ask(1, Codec::Dib, BOOM)]), Vec::new());
    let outcome = answer(&mut &input[..], &mut out, test_draw);
    assert_eq!(outcome, Ok(()));
    assert_eq!(
        replies(&out),
        [Reply::Failed {
            id: 1,
            why: Why::Internal
        }]
    );
    io::stdout().write_all(ANSWERED.as_bytes()).unwrap();
}

#[test]
fn draws_are_answered_in_order_until_the_input_ends() {
    let (dib, _) = tiny();
    let asks: Vec<Ask<'_>> = (10..15).map(|id| draw_ask(id, Codec::Dib, &dib)).collect();
    let (outcome, replies) = run(&frames(&asks));
    assert_eq!(outcome, Ok(()));
    let ids: Vec<u32> = replies.iter().map(Reply::id).collect();
    assert_eq!(ids, [10, 11, 12, 13, 14]);
}

#[test]
fn no_input_ends_cleanly_with_no_reply() {
    assert_eq!(run(&[]), (Ok(()), Vec::new()));
}

#[test]
fn nothing_after_bye_is_read() {
    let (dib, _) = tiny();
    let tail = frames(&[draw_ask(1, Codec::Dib, &dib)]);
    let input = [frames(&[Ask::Bye]), tail.clone()].concat();
    let mut rest = &input[..];
    let mut out = Vec::new();
    assert_eq!(serve(&mut rest, &mut out, drawing), Ok(()));
    assert_eq!((rest, out.len()), (&tail[..], 0));
}

#[test]
fn a_broken_frame_ends_the_loop_with_its_error() {
    let (dib, _) = tiny();
    let good = frames(&[draw_ask(1, Codec::Dib, &dib)]);
    assert_eq!(run(b"XXXX garbage frame").0, Err(IpcError::BadMagic));
    let cut = [good.clone(), good[..good.len() - 1].to_vec()].concat();
    let (outcome, replies) = run(&cut);
    assert_eq!(outcome, Err(IpcError::Short));
    assert_eq!(replies.len(), 1);
}

/// A writer whose stream is gone.
struct Gone;

impl Write for Gone {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::ErrorKind::BrokenPipe.into())
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::ErrorKind::BrokenPipe.into())
    }
}

#[test]
fn a_stream_that_cannot_be_written_ends_the_loop() {
    let (dib, _) = tiny();
    let input = frames(&[draw_ask(1, Codec::Dib, &dib)]);
    let outcome = serve(&mut &input[..], &mut Gone, drawing);
    assert_eq!(outcome, Err(IpcError::Io(io::ErrorKind::BrokenPipe)));
}
