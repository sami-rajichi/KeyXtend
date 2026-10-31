//! Sessions on their own threads over a scripted worker: a draw goes through and a dropped session stops its worker,
//! an idle worker stops by itself, a worker that never starts is gone, and exit codes read as notes.

use super::*;
use fake::{Act, Call, Hands, Script};
use kx_module_api::Redacted;
use kx_module_api::ports::ImageCodec;
use std::collections::HashSet;

/// How long a test waits for the session thread; a wait, never a sleep.
const WAIT: Duration = Duration::from_secs(10);

/// Test values; the real ones come from the clipboard module's settings.
const CFG: PictureConfig = PictureConfig {
    deadline_ms: 2_000,
    idle_s: 30,
    memory_mb: 256,
    max_pixels: 40_000_000,
    restart_ms: 1_000,
    restart_max_ms: 4_000,
};

/// The longest side the tests ask for.
const SIDE: u32 = 4;

fn ask() -> PictureAsk {
    PictureAsk {
        codec: ImageCodec::Dib,
        bytes: Redacted::new(vec![5; 6]),
        side: SIDE,
    }
}

/// Pictures over `script`, and a channel that hears each call to its workers.
fn pictures(script: &Script) -> (WorkerPictures<Script>, Receiver<Call>) {
    let (tell, told) = mpsc::channel();
    script.tell(tell);
    (
        WorkerPictures::new(script.clone(), Arc::new(Hands::default())),
        told,
    )
}

/// A wait for the exit code, as long as the deadline.
fn code() -> Call {
    let within = Duration::from_millis(CFG.deadline_ms.into());
    Call::Code { within }
}

/// Waits until the session's worker is ended.
fn ended(told: &Receiver<Call>) {
    while told.recv_timeout(WAIT).expect("the worker is ended") != Call::End {}
}

#[test]
fn a_session_draws_on_its_thread_and_its_drop_stops_the_worker() {
    let script = Script::default();
    let (pictures, told) = pictures(&script);
    let mut run = pictures.open(CFG);
    script.act(Act::Draw(3, 2));
    let drawn = run.draw(&ask()).unwrap();
    assert_eq!((drawn.width(), drawn.height()), (3, 2));
    drop(run);
    ended(&told);
    assert!(script.calls().ends_with(&[Call::Bye, code(), Call::End]));
}

#[test]
fn a_session_stops_an_idle_worker_by_itself() {
    let script = Script::default();
    let (pictures, told) = pictures(&script);
    let mut run = pictures.open(PictureConfig { idle_s: 0, ..CFG });
    run.draw(&ask()).unwrap();
    ended(&told);
    assert!(script.calls().contains(&Call::Bye));
    run.draw(&ask()).unwrap();
    assert_eq!(script.spawns(), 2);
}

#[test]
fn a_draw_whose_worker_never_starts_is_gone() {
    let script = Script::default();
    script.refuse(1);
    let (pictures, _told) = pictures(&script);
    let mut run = pictures.open(CFG);
    assert_eq!(run.draw(&ask()), Err(PictureError::Gone));
}

#[test]
fn exit_codes_read_as_notes() {
    let code = |c: u8| Some(u32::from(c));
    let codes = [code(CLEAN_EXIT), code(REFUSED_EXIT), Some(101), None];
    let notes: HashSet<_> = codes.into_iter().map(exit_note).collect();
    assert_eq!(notes.len(), codes.len());
    assert_eq!(exit_note(Some(u32::MAX)), exit_note(Some(101)));
}
