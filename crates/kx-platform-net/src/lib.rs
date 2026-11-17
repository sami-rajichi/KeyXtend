//! Worker-only HTTPS requests (ADR-0004, ADR-0043 decision 8, ADR-0047 decision 6): the OS-neutral `Http` port with
//! its redirect, range, header and fault rules, and `WinHttp`, its `WinHTTP` adapter. The uiAccess app never depends on
//! this crate.

// Backstop to the workspace lint: each unsafe block in the adapter still needs its own scoped allow.
#![deny(unsafe_code)]

mod call;
mod error;
#[cfg(any(test, feature = "fake"))]
pub mod fake;
mod follow;
mod range;
mod target;
#[cfg(windows)]
mod win;

pub use call::{Call, Field, SEND_CHUNK, Verb};
pub use error::NetError;
pub use range::Span;
pub use target::Target;
#[cfg(windows)]
pub use win::WinHttp;

use kx_ipc::STALL_S;
use std::borrow::Cow;
use std::fmt;

/// The user agent every request sends: the product's name, nothing about the user or the PC.
pub const AGENT: &str = "KeyXtend";
/// The most redirects one download follows; a model's host sends one or two.
pub const MAX_HOPS: usize = 5;
/// Milliseconds in a second.
const MS_PER_S: u32 = 1000;

/// One GET to one checked address, from byte `from` on; it never follows a redirect.
#[derive(Clone, Copy, Debug)]
pub struct Hop<'a> {
    /// The address.
    pub target: &'a Target,
    /// The first byte asked; 0 asks for the whole file.
    pub from: u64,
    /// Seconds without a byte, while connecting or reading, before the request is a stall; see `wait_ms`.
    pub stall_s: u16,
}

/// An answer's status and the headers the rules read, as the server sent them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Head {
    /// The HTTP status code.
    pub status: u16,
    /// `Location`, which a redirect names.
    pub location: Option<String>,
    /// `Content-Length`.
    pub length: Option<String>,
    /// `Content-Range`, which a partial answer names.
    pub range: Option<String>,
    /// The headers a call asked to keep that the answer holds, as names and values.
    pub extra: Vec<(String, String)>,
}

impl Head {
    /// The kept header `name`'s value, its name matched in any case.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        let named = |(n, _): &&(String, String)| n.eq_ignore_ascii_case(name);
        self.extra.iter().find(named).map(|(_, v)| v.as_str())
    }
}

/// One answer: its head and its body.
pub struct Reply {
    /// The status and headers.
    pub head: Head,
    /// The body, read as it arrives.
    pub body: Box<dyn Body>,
}

/// A body's bytes in order; it may move to another thread, and closes its request when dropped.
pub trait Body: Send {
    /// Reads the next bytes into `buf` and says how many; 0 means the body ended.
    ///
    /// # Errors
    /// `Stall` when no byte came within the hop's stall time, `Net` when the connection broke, `Internal` otherwise.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, NetError>;
}

/// One download: an address from the table, the hosts it may reach, and where to start.
#[derive(Clone, Copy, Debug)]
pub struct Get<'a> {
    /// The address; it must start with `kx_ipc::HTTPS` and name a listed host.
    pub url: &'a str,
    /// The only hosts the address and its redirects may name, in lower case.
    pub hosts: &'a [Cow<'a, str>],
    /// The first byte asked: the part's length on a resume, else 0.
    pub from: u64,
    /// Seconds without a byte before the request is a stall; see `wait_ms`.
    pub stall_s: u16,
}

/// The answer to a `Get` once its redirects are followed.
pub struct Answer {
    /// The address that answered.
    pub url: Target,
    /// Where the body starts in the file: `from`, or 0 when the server sent the whole file, so the part restarts.
    pub start: u64,
    /// The whole file's size, when the server said it.
    pub total: Option<u64>,
    /// The body from `start` on.
    pub body: Box<dyn Body>,
}

/// Sends requests; the worker's fetch and speech jobs hold one, and tests a fake.
pub trait Http: Send + Sync {
    /// Sends one GET and hands back the answer as it came, a redirect included.
    ///
    /// # Errors
    /// `Net` when the name, the connection or TLS failed, `Stall` when no byte came in time, `Server` for a head that
    /// cannot be read, `Internal` otherwise.
    fn send(&self, hop: &Hop<'_>) -> Result<Reply, NetError>;

    /// Sends one call as it is and hands back the answer as it came, a redirect included; use `call`.
    ///
    /// # Errors
    /// As `send`.
    fn deliver(&self, call: &Call<'_>) -> Result<Reply, NetError>;

    /// Sends `call` once its head passes `Call::check`; a redirect comes back unfollowed, so its headers and body
    /// reach one address only.
    ///
    /// # Errors
    /// `Header` before anything is sent for a call `check` refuses; else as `send`.
    fn call(&self, call: &Call<'_>) -> Result<Reply, NetError> {
        call.check()?;
        self.deliver(call)
    }

    /// Gets `ask.url` from `ask.from` on, following at most `MAX_HOPS` redirects to listed HTTPS hosts.
    ///
    /// # Errors
    /// Any `NetError`; an unlisted or non-HTTPS address is refused before it is asked.
    fn get(&self, ask: &Get<'_>) -> Result<Answer, NetError> {
        follow::get(self, ask)
    }
}

/// How long a connection or a read waits for a byte: `stall_s` held within `kx_ipc::STALL_S`, in milliseconds.
/// The floor matters: `WinHTTP` reads a wait of 0 as no limit at all.
#[must_use]
pub fn wait_ms(stall_s: u16) -> u32 {
    let held = stall_s.clamp(*STALL_S.start(), *STALL_S.end());
    u32::from(held) * MS_PER_S
}

impl fmt::Debug for Reply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Reply")
            .field("head", &self.head)
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for Answer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Answer")
            .field("url", &self.url)
            .field("start", &self.start)
            .field("total", &self.total)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests;
