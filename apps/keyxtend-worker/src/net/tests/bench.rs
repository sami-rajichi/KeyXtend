//! The bench a job's tests run on: a job, its requests' channel and its notes, the asks and notes they build, and two
//! servers that hold a request until the test lets it answer.

use super::*;
use crate::net::job::Job;
use crate::net::{Event, Notes};
use kx_ipc::{IpcError, NetAsk, NetFault, NetNote};
use kx_platform_net::{Call, Hop, Http, NetError, Reply};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

/// The longest a test waits for a request's outcome before it fails; nothing sleeps.
pub(super) const WAIT: Duration = Duration::from_secs(20);

impl Notes for Vec<NetNote> {
    fn tell(&mut self, note: NetNote) -> Result<(), IpcError> {
        self.push(note);
        Ok(())
    }
}

/// A job, the channel its requests tell on, and its notes.
pub(super) struct Bench {
    pub job: Job,
    pub rx: Receiver<Event>,
    pub out: Vec<NetNote>,
}

impl Bench {
    /// A job over `http` and `clock`.
    pub fn new(http: Arc<dyn Http>, clock: &Arc<Still>) -> Self {
        let (tx, rx) = mpsc::channel();
        let clock: Arc<dyn Clock> = clock.clone();
        Self {
            job: Job::new(Kit { http, clock }, tx),
            rx,
            out: Vec::new(),
        }
    }

    /// Takes `ask`.
    pub fn ask(&mut self, ask: NetAsk<'static>) {
        self.job.take(ask, &mut self.out).unwrap();
    }

    /// Hands the job one request's outcome.
    pub fn one(&mut self) {
        match self.rx.recv_timeout(WAIT).unwrap() {
            Event::Told(told) => self.job.told(told, &mut self.out).unwrap(),
            _ => panic!("only requests tell here"),
        }
    }

    /// Hands the job outcomes until it has told `n` notes.
    pub fn until(&mut self, n: usize) -> &[NetNote] {
        while self.out.len() < n {
            self.one();
        }
        &self.out
    }
}

/// Run `id`'s send ask.
pub(super) fn send(id: u32, call: NetCall<'static>, run: NetRun<'static>) -> NetAsk<'static> {
    NetAsk::Send { id, call, run }
}

/// A part of run `id`'s piece `piece`.
pub(super) fn part(id: u32, piece: u16, bytes: Vec<u8>) -> NetAsk<'static> {
    let bytes = Wiped::Kept(Zeroizing::new(bytes));
    NetAsk::Sound { id, piece, bytes }
}

/// Run `id`'s text for piece `index`.
pub(super) fn text(id: u32, index: u16, text: &str) -> NetNote {
    let text = Zeroizing::new(text.to_owned());
    NetNote::Text { id, index, text }
}

/// Run `id`'s fault.
pub(super) fn fault(id: u32, fault: NetFault) -> NetNote {
    NetNote::Fault { id, fault }
}

/// A bench over a fake doing `acts`, with the fake and the clock.
pub(super) fn faked(acts: impl IntoIterator<Item = Act>) -> (Bench, Arc<FakeHttp>, Arc<Still>) {
    let (http, clock) = (Arc::new(FakeHttp::new(acts)), Arc::new(Still::default()));
    (Bench::new(http.clone(), &clock), http, clock)
}

/// A server that answers each piece by its sound's fill byte: piece 1 at once, piece 0 once the test lets it.
pub(super) struct Swap(pub Mutex<Receiver<()>>);

impl Http for Swap {
    fn send(&self, _: &Hop<'_>) -> Result<Reply, NetError> {
        Err(NetError::Net)
    }

    fn deliver(&self, call: &Call<'_>) -> Result<Reply, NetError> {
        let first = call.body.windows(4).any(|w| w == [0xA0; 4]);
        if first {
            lock(&self.0).recv_timeout(WAIT).unwrap();
        }
        FakeHttp::new([said(if first { "zero" } else { "one" })]).call(call)
    }
}

/// A server that tells the test each call it starts, then answers only once the test lets it.
pub(super) struct Gate(Mutex<Sender<()>>, Mutex<Receiver<()>>);

impl Http for Gate {
    fn send(&self, _: &Hop<'_>) -> Result<Reply, NetError> {
        Err(NetError::Net)
    }

    fn deliver(&self, call: &Call<'_>) -> Result<Reply, NetError> {
        lock(&self.0).send(()).unwrap();
        lock(&self.1).recv_timeout(WAIT).unwrap();
        FakeHttp::new([said("x")]).call(call)
    }
}

/// A gate, the test's ends: calls started, and the release.
pub(super) fn gate() -> (Arc<Gate>, Receiver<()>, Sender<()>) {
    let ((started, seen), (free, freed)) = (mpsc::channel(), mpsc::channel());
    (
        Arc::new(Gate(Mutex::new(started), Mutex::new(freed))),
        seen,
        free,
    )
}
