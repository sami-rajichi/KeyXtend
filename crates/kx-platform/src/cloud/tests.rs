//! The cloud host over a scripted speech-net worker on a clock that moves only when a test moves it: the frames it
//! sends, the notes it tells and the bounds it keeps. Tests wait on channels, never sleep.

use super::*;
use crate::worker::fake::{Call, Hands, Script};
use kx_ipc::{NetAsk, NetFault, NetNote, SAMPLE_BYTES, SOUND_BYTES_PER_S};
use kx_module_api::ports::{CloudLimits, CloudShape, FormFields, HttpsUrl, JobLimits, KeyHeader};
use kx_module_api::{Redacted, SecretBytes, Zeroizing};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

mod bound;
mod hand;
mod key;
mod order;
mod props;
mod relay;
mod show;

/// How long a test waits for a note or a call; a wait, never a sleep.
const LONG: Duration = Duration::from_secs(10);
/// An example address.
const ADDRESS: &str = "https://stt.example.com/v1/audio/transcriptions";
/// The example's key test address.
const MODELS: &str = "https://stt.example.com/v1/models";
/// The made-up model the test row names.
const MODEL: &str = "made-up-model-v3";
/// The language code the test row sends.
const LANG: &str = "fr";
/// A transcript no output may show.
const SAID: &str = "bonjour-4417";
/// One test sample, little-endian.
const SAMPLE: [u8; SAMPLE_BYTES] = [0x11, 0x5a];
/// Bytes of hand-over sound in one second.
const PER_S: usize = SOUND_BYTES_PER_S;
/// The wait and the tries the test asks give, as the `[voice]` defaults do.
const WAIT_S: u16 = 60;
const TRIES: u8 = 2;
/// The job's run id: each run has a job of its own, whose first ask is 1.
const ID: u32 = 1;

/// What a run told, in order.
type Heard = Receiver<CloudNote>;
/// The host under test.
type Host = WorkerCloud<Script<NetFrames>>;

/// The job's limits as the `[voice]` defaults give them.
fn job() -> JobLimits {
    JobLimits {
        job_mb: 512,
        idle_min: 5,
        ask_ms: 3_000,
        load_s: 60,
        warm_s: 180,
        piece_wait_s: 120,
        restart_ms: 1_000,
        restart_max_ms: 30_000,
    }
}

/// Made-up limits, as a row might set them.
fn limits() -> CloudLimits {
    CloudLimits {
        max_mb: 25,
        max_s: 600,
        per_min: 20,
        at_once: 2,
    }
}

/// `bytes` of whole test samples as secret sound.
fn sound(bytes: usize) -> SecretBytes {
    Redacted::new(Zeroizing::new(SAMPLE.repeat(bytes / SAMPLE_BYTES)))
}

/// A run of pieces of these sizes for the made-up model in French.
fn ask(sizes: &[usize]) -> CloudAsk {
    CloudAsk {
        job: job(),
        shape: CloudShape::Form(FormFields {
            lang: "language".into(),
            list: false,
            fixed: Vec::new(),
            paywall: false,
            region: None,
        }),
        url: HttpsUrl::new(ADDRESS).expect("an https address"),
        header: KeyHeader::Bearer,
        key: Redacted::new(Zeroizing::new(key())),
        model: Some(Redacted::new(MODEL.to_owned())),
        lang: LANG.to_owned(),
        limits: limits(),
        wait_s: WAIT_S,
        tries: TRIES,
        pieces: sizes.iter().map(|&n| sound(n)).collect(),
    }
}

/// A key test of the made-up key.
fn key_test() -> CloudTest {
    CloudTest {
        job: job(),
        shape: CloudShape::Form(FormFields {
            lang: "language".into(),
            list: false,
            fixed: Vec::new(),
            paywall: false,
            region: None,
        }),
        url: HttpsUrl::new(MODELS).expect("an https address"),
        header: KeyHeader::Bearer,
        key: Redacted::new(Zeroizing::new(key())),
        wait_s: WAIT_S,
    }
}

/// A made-up key no output may show, built as the test runs so no key-like text sits in the source.
fn key() -> String {
    ["kx", "made", "up", "key"].join("-")
}

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
