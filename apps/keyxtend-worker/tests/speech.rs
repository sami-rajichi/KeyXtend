//! The `speech-net` job as a program over real pipes: a shape it does not build and a part out of order are refused
//! before any request, so no test reaches the network; it ends at goodbye, and fails a broken frame with one line.

#![allow(clippy::unwrap_used, reason = "a failed setup should fail the test")]

use kx_ipc::{
    CLEAN_EXIT, NET_JOB, NetAsk, NetCall, NetFault, NetHeader, NetLimits, NetNote, NetRun,
    NetShape, REFUSED_EXIT, Wiped,
};
use std::borrow::Cow;
use std::io::Write;
use std::process::{Child, Command, Stdio};

/// The worker program Cargo built for these tests.
const WORKER: &str = env!("CARGO_BIN_EXE_keyxtend-worker");

/// The speech job, over pipes.
fn speech_job() -> Child {
    let mut job = Command::new(WORKER);
    job.arg(NET_JOB)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    job.spawn().unwrap()
}

/// A call in `shape` with a made-up key.
fn call(shape: NetShape<'static>) -> NetCall<'static> {
    NetCall {
        shape,
        url: Cow::Borrowed("https://speech.example/v1/transcribe"),
        header: NetHeader::Bearer,
        key: Wiped::Lent(b"kx-test-key-not-real"),
        wait_s: 30,
    }
}

/// A run of two pieces of `bytes` each.
fn run(bytes: u32) -> NetRun<'static> {
    let limits = NetLimits {
        max_mb: 25,
        max_s: 600,
        per_min: 20,
        at_once: 1,
    };
    NetRun {
        model: Some(Cow::Borrowed("whisper-large-v3-turbo")),
        lang: Cow::Borrowed("fr"),
        limits,
        tries: 1,
        sizes: vec![bytes; 2],
    }
}

/// A shape the job does not build, a part for a piece not reached yet, an unknown id's cancel, then goodbye.
fn asks() -> [NetAsk<'static>; 5] {
    let form = NetShape::Form(kx_ipc::NetForm {
        lang: Cow::Borrowed("language"),
        list: false,
        fixed: Vec::new(),
        paywall: false,
        region: None,
    });
    let send = |id, call| NetAsk::Send {
        id,
        call,
        run: run(4),
    };
    let late = NetAsk::Sound {
        id: 2,
        piece: 1,
        bytes: Wiped::Lent(&[0, 0]),
    };
    let (eleven, form) = (send(1, call(NetShape::Eleven)), send(2, call(form)));
    [eleven, form, late, NetAsk::Cancel { id: 9 }, NetAsk::Bye]
}

#[test]
fn the_speech_job_refuses_without_the_network_until_bye() {
    let mut job = speech_job();
    for ask in &asks() {
        ask.write(job.stdin.as_mut().unwrap()).unwrap();
    }
    let out = job.wait_with_output().unwrap();
    let mut stdout = &out.stdout[..];
    let notes = [(); 2].map(|()| NetNote::read(&mut stdout).unwrap());
    let fault = |id, fault| NetNote::Fault { id, fault };
    assert_eq!(notes, [fault(1, NetFault::Range), fault(2, NetFault::Gone)]);
    assert!(stdout.is_empty(), "nothing more is told");
    let code = out.status.code();
    assert_eq!(code, Some(i32::from(CLEAN_EXIT)), "{:?}", out.stderr);
    assert!(out.stderr.is_empty());
}

#[test]
fn the_speech_job_fails_a_broken_frame_with_one_line() {
    let mut job = speech_job();
    let input = job.stdin.take().unwrap();
    { input }.write_all(b"not a frame at all").unwrap();
    let out = job.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(i32::from(REFUSED_EXIT)));
    assert!(out.stdout.is_empty());
    let text = String::from_utf8_lossy(&out.stderr);
    assert!(
        text.lines().count() == 1 && text.ends_with('\n'),
        "{text:?}"
    );
}
