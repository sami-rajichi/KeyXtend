//! The program itself: a refused command line fails with one line on stderr, and the media and vault jobs answer
//! over real stdin and stdout until goodbye.

#![allow(clippy::unwrap_used, reason = "a failed setup should fail the test")]

use kx_ipc::{
    Ask, BoxPart, CLEAN_EXIT, Codec, IpcError, MAX_PIXELS_FLAG, MEDIA_JOB, MIN_BOX_WORK,
    NOTEBOOK_PART, REFUSED_EXIT, Reply, VAULT_JOB, VaultAsk, VaultReply, Why, Wiped,
};
use std::borrow::Cow;
use std::io::Write;
use std::process::{Child, Command, Output, Stdio};

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
        &[VAULT_JOB, MEDIA_JOB],
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

/// A 1 by 1 RGB PNG of red 1, green 2, blue 3.
fn tiny_png() -> Vec<u8> {
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, 1, 1);
    encoder.set_color(png::ColorType::Rgb);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&[1, 2, 3]).unwrap();
    writer.finish().unwrap();
    out
}

#[test]
fn the_media_job_makes_a_dib_from_a_png() {
    let mut input = Vec::new();
    for (id, png) in [(1, tiny_png()), (2, b"junk".to_vec())] {
        let bytes = Cow::Owned(png);
        let ask = Ask::Dib {
            id,
            max_bytes: 1_000,
            bytes,
        };
        ask.write(&mut input).unwrap();
    }
    let out = run(&MEDIA, &input);
    assert!(exited(&out, CLEAN_EXIT), "{:?}", out.stderr);
    let mut stdout = &out.stdout[..];
    let Ok(Reply::Dib { id: 1, bytes }) = Reply::read(&mut stdout) else {
        panic!("a DIB for ask 1");
    };
    assert_eq!(
        (bytes.len(), &bytes[..4], &bytes[40..]),
        (44, &[40, 0, 0, 0][..], &[3, 2, 1, 255][..])
    );
    let refused = Reply::Failed {
        id: 2,
        why: Why::Refused,
    };
    assert_eq!(Reply::read(&mut stdout), Ok(refused));
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

/// The vault job, asked over pipes, its stderr kept to check.
fn vault_job() -> Child {
    let mut job = Command::new(WORKER);
    job.arg(VAULT_JOB)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    job.spawn().unwrap()
}

/// Writes `ask` to the vault job and reads its answer.
fn ask_vault(job: &mut Child, ask: &VaultAsk<'_>) -> VaultReply {
    ask.write(job.stdin.as_mut().unwrap()).unwrap();
    VaultReply::read(job.stdout.as_mut().unwrap()).unwrap()
}

/// The padlock the job makes under `password`: its recipient and its locked identity.
fn padlock(job: &mut Child, password: &[u8]) -> (String, Vec<u8>) {
    let password = Wiped::Lent(password);
    let ask = VaultAsk::Padlock {
        id: 1,
        work: MIN_BOX_WORK,
        password,
    };
    let VaultReply::Padlock {
        recipient, locked, ..
    } = ask_vault(job, &ask)
    else {
        panic!("a padlock for ask 1");
    };
    (recipient, locked)
}

/// The box the job seals `notes` into, with no password.
fn seal(job: &mut Child, recipient: &str, locked: &[u8], notes: &[u8]) -> Vec<u8> {
    let part = BoxPart {
        name: Cow::Borrowed(NOTEBOOK_PART),
        bytes: Wiped::Lent(notes),
    };
    let (recipient, locked) = (Cow::Borrowed(recipient), Cow::Borrowed(locked));
    let ask = VaultAsk::Seal {
        id: 2,
        recipient,
        locked,
        parts: vec![part],
    };
    let VaultReply::Sealed { bytes, .. } = ask_vault(job, &ask) else {
        panic!("a box for ask 2");
    };
    bytes
}

#[test]
fn the_vault_job_seals_and_opens_a_box_one_ask_at_a_time_until_bye() {
    let (mut job, password) = (vault_job(), b"plain words");
    let (recipient, locked) = padlock(&mut job, password);
    let bytes = seal(&mut job, &recipient, &locked, b"notes");
    let password = Wiped::Lent(password);
    let open = VaultAsk::Open {
        id: 3,
        password,
        bytes: Cow::Borrowed(&bytes),
    };
    let VaultReply::Opened { parts, .. } = ask_vault(&mut job, &open) else {
        panic!("an opened box for ask 3");
    };
    assert_eq!(&*parts[0].bytes, b"notes");
    let password = Wiped::Lent(b"two fingers");
    let check = VaultAsk::Check {
        id: 4,
        password,
        locked: Cow::Borrowed(&locked),
    };
    assert_eq!(
        ask_vault(&mut job, &check),
        VaultReply::Checked { id: 4, yes: false }
    );
    VaultAsk::Bye.write(job.stdin.as_mut().unwrap()).unwrap();
    let out = job.wait_with_output().unwrap();
    assert!(
        exited(&out, CLEAN_EXIT) && out.stdout.is_empty() && out.stderr.is_empty(),
        "{:?} {:?}",
        out.status,
        out.stderr
    );
}
