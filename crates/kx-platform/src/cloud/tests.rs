//! The cloud host over a scripted speech-net worker on a clock that moves only when a test moves it: the frames it
//! sends, the notes it tells and the bounds it keeps. Tests wait on channels, never sleep.

use super::*;
use crate::worker::fake::{Call, Hands, Script};
use kx_ipc::{NetAsk, NetFault, NetNote, SAMPLE_BYTES, SOUND_BYTES_PER_S};
use kx_module_api::ports::{CloudShape, FormFields, KeyHeader};
use kx_module_api::{Redacted, Zeroizing};
use kx_platform_fake::cloud_suite::{
    ADDRESS, LANG, MODEL, MODELS, TRIES, WAIT_S, ask, job, key, key_test, limits, sound,
};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

mod bound;
mod hand;
mod key;
mod order;
mod props;
mod relay;
mod show;
mod suite;

/// How long a test waits for a note or a call; a wait, never a sleep.
const LONG: Duration = Duration::from_secs(10);
/// A transcript no output may show.
const SAID: &str = "bonjour-4417";
/// Bytes of hand-over sound in one second.
const PER_S: usize = SOUND_BYTES_PER_S;
/// The job's run id: each run has a job of its own, whose first ask is 1.
const ID: u32 = 1;

/// What a run told, in order.
type Heard = Receiver<CloudNote>;
/// The host under test.
type Host = WorkerCloud<Script<NetFrames>>;

/// A host over a fresh script, on a clock the test moves.
fn host() -> (Host, Script<NetFrames>, Hands) {
    let (script, clock) = (Script::default(), Hands::default());
    let host = WorkerCloud::new(script.clone(), Arc::new(clock.clone()));
    (host, script, clock)
}

/// Sends `ask` and gives the run and what it tells.
fn sent(host: &Host, ask: CloudAsk) -> (Box<dyn CloudRun>, Heard) {
    let (said, heard) = mpsc::channel();
    let tell = Box::new(move |note| drop(said.send(note)));
    (host.send(ask, tell).expect("the run starts"), heard)
}

/// The next `n` notes told.
fn next(heard: &Heard, n: usize) -> Vec<CloudNote> {
    let one = |_| heard.recv_timeout(LONG).expect("told in time");
    (0..n).map(one).collect()
}

/// The net asks the worker took, in order.
fn nets(script: &Script<NetFrames>) -> Vec<NetAsk<'static>> {
    let net = |call| match call {
        Call::Net(ask) => Some(ask),
        _ => None,
    };
    script.calls().into_iter().filter_map(net).collect()
}

/// Waits until the worker took `ask`.
fn took(called: &Receiver<Call>, ask: &NetAsk<'static>) {
    let want = Call::Net(ask.clone());
    while called.recv_timeout(LONG).expect("the ask is sent") != want {}
}

/// The job's text for piece `index`.
fn text(index: u16, said: &str) -> NetNote {
    NetNote::Text {
        id: ID,
        index,
        text: Zeroizing::new(said.to_owned()),
    }
}

/// The port's text for piece `index`.
fn piece(index: u16, said: &str) -> CloudNote {
    CloudNote::Piece {
        index,
        text: Redacted::new(Zeroizing::new(said.to_owned())),
    }
}

/// The job's fault.
const fn fault(fault: NetFault) -> NetNote {
    NetNote::Fault { id: ID, fault }
}
