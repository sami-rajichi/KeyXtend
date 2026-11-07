//! A session runs each call on its own named thread over a scripted worker, and its drop stops the worker; exit codes
//! read as notes.

use super::*;
use fake::{Act, Call, Hands, Script};
use kx_ipc::{Ask, Reply, Why};
use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::thread;

/// How long a test waits for the session thread; a wait, never a sleep.
const WAIT: Duration = Duration::from_secs(10);
/// The session thread's name in the tests.
const THREAD: &str = "kx-test-session";

/// Test values; the real ones come from each port's config.
const TIMES: Times = Times {
    deadline_ms: 2_000,
    idle_s: 30,
    restart_ms: 1_000,
    restart_max_ms: 4_000,
};

/// A session over `script`, and a channel that hears each call to its workers.
fn session(
    script: &Script<MediaFrames>,
) -> (Session<Script<MediaFrames>, MediaFrames>, Receiver<Call>) {
    let (tell, told) = mpsc::channel();
    script.tell(tell);
    let launch = Launch {
        args: Vec::new(),
        memory_mb: 1,
    };
    let clock = Arc::new(Hands::default());
    let host = Host::new(Arc::new(script.clone()), clock, TIMES, launch);
    (Session::open(THREAD, host), told)
}

/// One DIB ask on the session thread: that thread's name and the reply's id.
fn ask(
    session: &Session<Script<MediaFrames>, MediaFrames>,
) -> Option<(Option<String>, Result<u32, Lost>)> {
    session.run(|host| {
        let name = thread::current().name().map(str::to_owned);
        let make = |id| {
            let bytes = Cow::Borrowed(&[1_u8][..]);
            let max_bytes = 1;
            Ok(Ask::Dib {
                id,
                max_bytes,
                bytes,
            })
        };
        let answer = host.call(make, |reply: Reply| Some(Ok(reply.id())));
        (name, answer)
    })
}

#[test]
fn a_session_runs_each_call_on_its_named_thread_and_its_drop_stops_the_worker() {
    let script = Script::default();
    let (session, told) = session(&script);
    script.act(Act::Fail(Why::Refused));
    let name = Some(THREAD.to_owned());
    assert_eq!(ask(&session), Some((name.clone(), Ok(1))));
    assert_eq!(ask(&session), Some((name, Ok(2))));
    assert_eq!(script.spawns(), 1);
    drop(session);
    while told.recv_timeout(WAIT).expect("the worker is ended") != Call::End {}
    let within = Duration::from_millis(TIMES.deadline_ms.into());
    assert!(
        script
            .calls()
            .ends_with(&[Call::Bye, Call::Code { within }, Call::End])
    );
}

#[test]
fn exit_codes_read_as_notes() {
    let code = |c: u8| Some(u32::from(c));
    let codes = [code(CLEAN_EXIT), code(REFUSED_EXIT), Some(101), None];
    let notes: HashSet<_> = codes.into_iter().map(exit_note).collect();
    assert_eq!(notes.len(), codes.len());
    assert_eq!(exit_note(Some(u32::MAX)), exit_note(Some(101)));
}
