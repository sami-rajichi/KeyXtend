//! The program itself: a refused command line fails with one line on stderr, and the media job answers over real
//! stdin and stdout until goodbye.

#![allow(clippy::unwrap_used, reason = "a failed setup should fail the test")]

use kx_ipc::{
    Ask, CLEAN_EXIT, Codec, IpcError, MAX_PIXELS_FLAG, MEDIA_JOB, REFUSED_EXIT, Reply, Why,
};
use std::borrow::Cow;
use std::io::Write;
use std::process::{Command, Output, Stdio};

/// The worker program Cargo built for these tests.
const WORKER: &str = env!("CARGO_BIN_EXE_keyxtend-worker");
/// The media job's command line.
const MEDIA: [&str; 3] = [MEDIA_JOB, MAX_PIXELS_FLAG, "100"];

fn run(args: &[&str], input: &[u8]) -> Output {
    let mut child = Command::new(WORKER)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

/// A 1 by 1 24-bit DIB of red 1, green 2, blue 3.
fn tiny_dib() -> Vec<u8> {
    let mut head = [0_u8; 40];
    head[0] = 40;
    head[4] = 1;
    head[8] = 1;
    head[12] = 1;
    head[14] = 24;
    [&head[..], &[3, 2, 1, 0]].concat()
}

fn draw(id: u32, codec: Codec, bytes: &[u8]) -> Ask<'_> {
    Ask::Draw {
        id,
        codec,
        side: 4,
        bytes: Cow::Borrowed(bytes),
    }
}

fn one_line(stderr: &[u8]) -> bool {
    let text = String::from_utf8_lossy(stderr);
    text.lines().count() == 1 && text.ends_with('\n')
}

/// Whether the program ended with `code`, as the host reads it.
fn exited(out: &Output, code: u8) -> bool {
    out.status.code() == Some(i32::from(code))
}

#[test]
fn a_refused_command_line_fails_with_one_line() {
    let cases = [
        &[][..],
        &["voice"],
        &["ocr"],
        &["update"],
        &[MEDIA_JOB],
        &[MEDIA_JOB, MAX_PIXELS_FLAG, "0"],
    ];
    for args in cases {
        let out = run(args, b"");
        assert!(exited(&out, REFUSED_EXIT), "{args:?}: {:?}", out.status);
        assert!(out.stdout.is_empty(), "{args:?}");
        assert!(one_line(&out.stderr), "{args:?}: {:?}", out.stderr);
    }
}

#[test]
fn the_media_job_answers_until_bye() {
    let dib = tiny_dib();
    let mut input = Vec::new();
    for ask in [
        draw(1, Codec::Dib, &dib),
        draw(2, Codec::Png, b"junk"),
        Ask::Bye,
    ] {
        ask.write(&mut input).unwrap();
    }
    let out = run(&MEDIA, &input);
    assert!(exited(&out, CLEAN_EXIT), "{:?}", out.stderr);
    assert!(out.stderr.is_empty());
    let mut stdout = &out.stdout[..];
    let pixels = Reply::Pixels {
        id: 1,
        width: 1,
        height: 1,
        rgba: vec![1, 2, 3, 255],
    };
    assert_eq!(Reply::read(&mut stdout), Ok(pixels));
    assert_eq!(
        Reply::read(&mut stdout),
        Ok(Reply::Failed {
            id: 2,
            why: Why::Refused
        })
    );
    assert_eq!(Reply::read(&mut stdout), Err(IpcError::Closed));
}

#[test]
fn the_end_of_input_ends_the_job_cleanly() {
    let out = run(&MEDIA, b"");
    assert!(exited(&out, CLEAN_EXIT));
    assert!(out.stdout.is_empty() && out.stderr.is_empty());
}

#[test]
fn a_broken_frame_fails_the_job_with_one_line() {
    let out = run(&MEDIA, b"not a frame at all");
    assert!(exited(&out, REFUSED_EXIT));
    assert!(out.stdout.is_empty());
    assert!(one_line(&out.stderr), "{:?}", out.stderr);
}
