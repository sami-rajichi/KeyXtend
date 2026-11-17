//! One request with headers and maybe a body, as the `speech-net` job sends (ADR-0047 decision 6): checked before it
//! is sent, and never followed to another address. A header's value may be a key, so `Debug` shows names only.

use crate::{NetError, Target};
use kx_ipc::rules;
use std::fmt;

/// The one byte a header value may hold besides visible ASCII.
const SPACE: u8 = b' ';

/// A request's verb.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Verb {
    /// Reads, with no body.
    Get,
    /// Sends a body.
    Post,
}

impl Verb {
    /// The verb as HTTP writes it.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
        }
    }
}

/// A header a call sends; its value may be a key.
#[derive(Clone, Copy)]
pub struct Field<'a> {
    /// The name: ASCII letters, digits, `-` and `_` (`rules::token`), never empty.
    pub name: &'a str,
    /// The value: visible ASCII and spaces, so it never breaks the request's head.
    pub value: &'a [u8],
}

/// One request to one checked address with its headers and maybe a body; a redirect comes back unfollowed.
#[derive(Clone, Copy)]
pub struct Call<'a> {
    /// The address.
    pub target: &'a Target,
    /// The verb; a GET sends no body.
    pub verb: Verb,
    /// The headers, in order.
    pub fields: &'a [Field<'a>],
    /// The body, sent as it is.
    pub body: &'a [u8],
    /// The answer's headers to keep in `Head::extra`, by name (`rules::token`).
    pub keep: &'a [&'a str],
    /// Seconds without a byte, while connecting, sending or reading, before the call is a stall; see `wait_ms`.
    pub stall_s: u16,
}

impl Call<'_> {
    /// Refuses a call whose head would be misread, or a GET with a body.
    ///
    /// # Errors
    /// `Header` for an empty or non-token header or kept name, a value byte that is not visible ASCII or a space, or a
    /// GET with a body.
    pub fn check(&self) -> Result<(), NetError> {
        let name = |n: &str| !n.is_empty() && rules::token(n).is_ok();
        let value = |v: &[u8]| v.iter().all(|b| b.is_ascii_graphic() || *b == SPACE);
        let fields = self.fields.iter().all(|f| name(f.name) && value(f.value));
        let body = self.verb == Verb::Post || self.body.is_empty();
        if fields && body && self.keep.iter().all(|k| name(k)) {
            Ok(())
        } else {
            Err(NetError::Header)
        }
    }
}

impl fmt::Debug for Field<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Field")
            .field("name", &self.name)
            .field("len", &self.value.len())
            .finish()
    }
}

impl fmt::Debug for Call<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Call")
            .field("verb", &self.verb)
            .field("fields", &self.fields)
            .field("body", &self.body.len())
            .field("keep", &self.keep)
            .field("stall_s", &self.stall_s)
            .finish_non_exhaustive()
    }
}
