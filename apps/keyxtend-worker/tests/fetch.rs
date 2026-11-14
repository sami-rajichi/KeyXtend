//! The fetch job as a program over real pipes: it refuses an address whose host the ask does not list before any
//! request, so no test reaches the network, ends at goodbye, and fails a broken frame with one line.

#![allow(clippy::unwrap_used, reason = "a failed setup should fail the test")]

use kx_ipc::{
    CLEAN_EXIT, FETCH_JOB, FetchAsk, FetchFault, FetchNote, REFUSED_EXIT, SHA256_BYTES, STALL_S,
};
use std::borrow::Cow;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Child, Command, Stdio};

/// The worker program Cargo built for these tests.
const WORKER: &str = env!("CARGO_BIN_EXE_keyxtend-worker");

/// The fetch job, over pipes.
fn fetch_job() -> Child {
    let mut job = Command::new(WORKER);
    job.arg(FETCH_JOB)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    job.spawn().unwrap()
}

#[test]
fn the_fetch_job_refuses_an_unlisted_host_without_the_network_until_bye() {
    let name = format!("kx-fetch-cli-{}", std::process::id());
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    fs::create_dir_all(&dir).unwrap();
    let (file, total) = (dir.join("m.gguf"), 4);
    let get = FetchAsk::Get {
        id: 3,
        urls: vec![Cow::Borrowed("https://elsewhere.example/m.gguf")],
        hosts: vec![Cow::Borrowed("models.example")],
        path: Cow::Owned(file.to_string_lossy().into_owned()),
        size: total,
        sha256: [0; SHA256_BYTES],
        stall_s: *STALL_S.start(),
    };
    let mut job = fetch_job();
    get.write(job.stdin.as_mut().unwrap()).unwrap();
    let notes = [(); 2].map(|()| FetchNote::read(job.stdout.as_mut().unwrap()));
    FetchAsk::Bye.write(job.stdin.as_mut().unwrap()).unwrap();
    let out = job.wait_with_output().unwrap();
    let left = (file.exists(), dir.join("m.gguf.part").exists());
    fs::remove_dir_all(&dir).unwrap();
    let fault = FetchFault::Redirect;
    let want = [
        Ok(FetchNote::Progress {
            id: 3,
            got: 0,
            total,
        }),
        Ok(FetchNote::Fault { id: 3, fault }),
    ];
    assert_eq!(notes, want, "the empty part, then the refusal");
    let clean = out.status.code() == Some(i32::from(CLEAN_EXIT));
    assert!(clean, "{:?} {:?}", out.status, out.stderr);
    assert!(out.stdout.is_empty() && out.stderr.is_empty());
    assert_eq!(left, (false, false), "no file and no empty part");
}

#[test]
fn the_fetch_job_fails_a_broken_frame_with_one_line() {
    let mut job = fetch_job();
    job.stdin
        .take()
        .unwrap()
        .write_all(b"not a frame at all")
        .unwrap();
    let out = job.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(i32::from(REFUSED_EXIT)));
    assert!(out.stdout.is_empty());
    let text = String::from_utf8_lossy(&out.stderr);
    assert!(
        text.lines().count() == 1 && text.ends_with('\n'),
        "{text:?}"
    );
}
