//! A scripted `Http` for tests: each request takes the next act and is kept in order. It never sleeps or touches the
//! network; a quiet step at or past the hop's `wait_ms` reads as a stall, as `WinHTTP`'s receive timeout does.

use crate::follow::FOUND;
use crate::range::{DASH, OK, OVER, PARTIAL, UNIT};
use crate::{Body, Head, Hop, Http, NetError, Reply, wait_ms};
use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard, PoisonError};

/// What the fake server does with one request.
#[derive(Clone, Debug)]
pub enum Act {
    /// Answers with this head and a body of these steps.
    Answer(Head, Vec<Step>),
    /// Fails before any answer, as a refused connection or a failed TLS handshake does.
    Fail(NetError),
}

/// One step of a scripted body.
#[derive(Clone, Debug)]
pub enum Step {
    /// These bytes arrive.
    Bytes(Vec<u8>),
    /// No byte comes for this many milliseconds.
    Quiet(u32),
    /// The read fails, as a cut connection does.
    Fail(NetError),
}

/// A request the fake was sent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seen {
    /// The address, as `Target` writes it.
    pub url: String,
    /// The first byte asked.
    pub from: u64,
    /// The stall time asked.
    pub stall_s: u16,
}

/// The scripted server; with no act left, a request fails as `Net`.
#[derive(Debug, Default)]
pub struct FakeHttp {
    /// The acts still to come.
    acts: Mutex<VecDeque<Act>>,
    /// Every request so far.
    seen: Mutex<Vec<Seen>>,
}

impl FakeHttp {
    /// A server that does `acts` in order.
    pub fn new(acts: impl IntoIterator<Item = Act>) -> Self {
        let acts = Mutex::new(acts.into_iter().collect());
        Self {
            acts,
            seen: Mutex::default(),
        }
    }

    /// Queues one more act.
    pub fn push(&self, act: Act) {
        lock(&self.acts).push_back(act);
    }

    /// Every request so far, in order.
    #[must_use]
    pub fn seen(&self) -> Vec<Seen> {
        lock(&self.seen).clone()
    }
}

impl Http for FakeHttp {
    fn send(&self, hop: &Hop<'_>) -> Result<Reply, NetError> {
        let (url, from, stall_s) = (hop.target.to_string(), hop.from, hop.stall_s);
        lock(&self.seen).push(Seen { url, from, stall_s });
        match lock(&self.acts).pop_front() {
            Some(Act::Answer(head, steps)) => {
                let body = FakeBody {
                    steps: steps.into(),
                    wait: wait_ms(stall_s),
                };
                Ok(Reply {
                    head,
                    body: Box::new(body),
                })
            }
            Some(Act::Fail(e)) => Err(e),
            None => Err(NetError::Net),
        }
    }
}

impl Act {
    /// A whole-file answer holding `body`.
    #[must_use]
    pub fn whole(body: &[u8]) -> Self {
        Self::Answer(Head::whole(body.len()), vec![Step::Bytes(body.to_vec())])
    }

    /// A partial answer holding `body`, from byte `first` of a file of `total` bytes.
    #[must_use]
    pub fn part(body: &[u8], first: u64, total: u64) -> Self {
        let last = first.saturating_add(body.len() as u64).saturating_sub(1);
        let head = Head {
            status: PARTIAL,
            length: Some(body.len().to_string()),
            range: Some(format!("{UNIT} {first}{DASH}{last}{OVER}{total}")),
            ..Head::default()
        };
        Self::Answer(head, vec![Step::Bytes(body.to_vec())])
    }

    /// A redirect to `to`.
    #[must_use]
    pub fn moved(to: &str) -> Self {
        Self::Answer(Head::moved(to), Vec::new())
    }

    /// An answer with `status` and an empty body.
    #[must_use]
    pub fn status(status: u16) -> Self {
        Self::Answer(
            Head {
                status,
                ..Head::default()
            },
            Vec::new(),
        )
    }
}

impl Head {
    /// A whole-file head for `len` bytes.
    #[must_use]
    pub fn whole(len: usize) -> Self {
        Self {
            status: OK,
            length: Some(len.to_string()),
            ..Self::default()
        }
    }

    /// A redirect head naming `to`.
    #[must_use]
    pub fn moved(to: &str) -> Self {
        Self {
            status: FOUND,
            location: Some(to.to_owned()),
            ..Self::default()
        }
    }
}

/// A scripted body.
struct FakeBody {
    /// The steps still to come.
    steps: VecDeque<Step>,
    /// The hop's wait, in ms; a quiet this long is a stall.
    wait: u32,
}

impl Body for FakeBody {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, NetError> {
        loop {
            match self.steps.pop_front() {
                None => return Ok(0),
                Some(Step::Bytes(mut bytes)) => {
                    let n = bytes.len().min(buf.len());
                    buf[..n].copy_from_slice(&bytes[..n]);
                    bytes.drain(..n);
                    if !bytes.is_empty() {
                        self.steps.push_front(Step::Bytes(bytes));
                    }
                    if n > 0 || buf.is_empty() {
                        return Ok(n);
                    }
                }
                Some(Step::Quiet(ms)) if ms >= self.wait => return Err(NetError::Stall),
                Some(Step::Quiet(_)) => {}
                Some(Step::Fail(e)) => return Err(e),
            }
        }
    }
}

/// `m`'s guard, even after a panic elsewhere.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}
