//! A scripted `Http` for tests that never sleeps or touches the network: each request takes the next act in order, and
//! a quiet step at or past `wait_ms` reads as a stall. A call is checked first, as `WinHttp` checks it, then written in
//! chunks that end once it is stopped, and its answer keeps only the headers it asked for.

use crate::follow::FOUND;
use crate::range::{DASH, OK, OVER, PARTIAL, UNIT};
use crate::{Body, Call, Head, Hop, Http, NetError, Reply, Verb, wait_ms};
use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

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

/// A call the fake was sent; `Debug` shows header names and lengths, never a value or the body.
#[derive(Clone, PartialEq, Eq)]
pub struct Called {
    /// The address, as `Target` writes it.
    pub url: String,
    /// The verb.
    pub verb: Verb,
    /// The headers, in order: names and values.
    pub fields: Vec<(String, Vec<u8>)>,
    /// The body.
    pub body: Vec<u8>,
    /// The headers it asked to keep.
    pub keep: Vec<String>,
    /// The stall time asked.
    pub stall_s: u16,
}

/// A test's hook on each body chunk.
#[derive(Clone)]
struct Hook(Arc<dyn Fn(usize) + Send + Sync>);

impl fmt::Debug for Hook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Hook")
    }
}

/// The scripted server; with no act left, a request fails as `Net`.
#[derive(Debug, Default)]
pub struct FakeHttp {
    /// The acts still to come.
    acts: Mutex<VecDeque<Act>>,
    /// Every request so far.
    seen: Mutex<Vec<Seen>>,
    /// Every call so far.
    called: Mutex<Vec<Called>>,
    /// Run after each body chunk a call writes, with the count so far.
    hook: Mutex<Option<Hook>>,
}

impl FakeHttp {
    /// A server that does `acts` in order.
    pub fn new(acts: impl IntoIterator<Item = Act>) -> Self {
        let acts = Mutex::new(acts.into_iter().collect());
        Self {
            acts,
            seen: Mutex::default(),
            called: Mutex::default(),
            hook: Mutex::default(),
        }
    }

    /// Runs `hook` after each body chunk a call writes, with the count of chunks so far, so a test can act mid-upload.
    pub fn on_chunk(&self, hook: impl Fn(usize) + Send + Sync + 'static) {
        *lock(&self.hook) = Some(Hook(Arc::new(hook)));
    }

    /// Every call so far, in order.
    #[must_use]
    pub fn called(&self) -> Vec<Called> {
        lock(&self.called).clone()
    }

    /// The next act as a reply whose body stalls at `wait`.
    fn next(&self, wait: u32) -> Result<Reply, NetError> {
        match lock(&self.acts).pop_front() {
            Some(Act::Answer(head, steps)) => {
                let body = FakeBody {
                    steps: steps.into(),
                    wait,
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
        self.next(wait_ms(stall_s))
    }

    fn deliver(&self, call: &Call<'_>) -> Result<Reply, NetError> {
        call.check()?;
        let field = |f: &crate::Field<'_>| (f.name.to_owned(), f.value.to_vec());
        let index = {
            let mut called = lock(&self.called);
            called.push(Called {
                url: call.target.to_string(),
                verb: call.verb,
                fields: call.fields.iter().map(field).collect(),
                body: Vec::new(),
                keep: call.keep.iter().map(|k| (*k).to_owned()).collect(),
                stall_s: call.stall_s,
            });
            called.len() - 1
        };
        let hook = lock(&self.hook).clone();
        let mut chunks = 0;
        call.upload(|chunk| {
            lock(&self.called)[index].body.extend_from_slice(chunk);
            chunks += 1;
            if let Some(Hook(hook)) = &hook {
                hook(chunks);
            }
            Ok(())
        })?;
        let mut reply = self.next(wait_ms(call.stall_s))?;
        let asked = |(n, _): &(String, String)| call.keep.iter().any(|k| k.eq_ignore_ascii_case(n));
        reply.head.extra.retain(asked);
        Ok(reply)
    }
}

impl fmt::Debug for Called {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<_> = self.fields.iter().map(|(n, v)| (n, v.len())).collect();
        f.debug_struct("Called")
            .field("verb", &self.verb)
            .field("fields", &names)
            .field("body", &self.body.len())
            .field("keep", &self.keep)
            .field("stall_s", &self.stall_s)
            .finish_non_exhaustive()
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
        Self::with(status, &[])
    }

    /// An answer with `status` and `body`.
    #[must_use]
    pub fn with(status: u16, body: &[u8]) -> Self {
        let head = Head {
            status,
            ..Head::default()
        };
        Self::Answer(head, vec![Step::Bytes(body.to_vec())])
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
