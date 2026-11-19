//! A request shape (ADR-0047 decision 5): one adapter per shape builds a piece's request, or a key's test, from the
//! call and the run, and reads the reply into text or a fault; the job holds no table. `ElevenLabs` and Gemini each
//! come as one more file and one more arm of `of`.

use super::form::Form;
use kx_ipc::{NetCall, NetFault, NetHeader, ShapeCode};
use kx_platform_net::{Head, Verb};
use zeroize::Zeroizing;

/// The header a bearer key goes in.
const AUTHORIZATION: &str = "Authorization";
/// What comes before a bearer key.
const BEARER: &[u8] = b"Bearer ";

/// What one request sends: its verb, headers, body and the answer's headers to keep.
pub struct Request {
    /// The verb.
    pub verb: Verb,
    /// The headers in order, as names and values; the key's value is wiped when dropped.
    pub fields: Vec<(String, Zeroizing<Vec<u8>>)>,
    /// The body, wiped when dropped, as it holds the sound.
    pub body: Zeroizing<Vec<u8>>,
    /// The answer's headers the shape reads.
    pub keep: &'static [&'static str],
}

/// One piece as a shape sends it.
pub struct Piece<'a> {
    /// Where it goes and how the key goes with it.
    pub call: &'a NetCall<'a>,
    /// The model's id, when the run names one.
    pub model: Option<&'a str>,
    /// The model's code for the language.
    pub lang: &'a str,
    /// The piece as a WAV file.
    pub wav: &'a [u8],
}

/// What a reply says.
pub enum Verdict<T> {
    /// What was asked: a piece's text, or that a key works.
    Said(T),
    /// Too many requests: wait this many seconds once, when the service says how long.
    Busy(Option<u32>),
    /// A fault.
    Fault(NetFault),
}

/// One request shape.
pub trait Shape: Send + Sync {
    /// The request that sends `piece`.
    ///
    /// # Errors
    /// `Range` for a value the shape cannot carry, `Gone` when memory runs out.
    fn build(&self, piece: &Piece<'_>) -> Result<Request, NetFault>;

    /// The request that tests `call`'s key, with no sound.
    ///
    /// # Errors
    /// As `build`.
    fn test(&self, call: &NetCall<'_>) -> Result<Request, NetFault>;

    /// What a piece's reply over `call` with `head` and `body` says.
    fn read(&self, call: &NetCall<'_>, head: &Head, body: &[u8]) -> Verdict<Zeroizing<String>>;

    /// What a key test's reply over `call` with `head` and `body` says.
    fn tested(&self, call: &NetCall<'_>, head: &Head, body: &[u8]) -> Verdict<()>;
}

/// The adapter for `code`; none for a shape this job does not build yet.
pub fn of(code: ShapeCode) -> Option<&'static dyn Shape> {
    match code {
        ShapeCode::Form => Some(&Form),
        ShapeCode::Eleven | ShapeCode::Gemini => None,
    }
}

/// The header that carries `call`'s key, its value wiped when dropped.
///
/// # Errors
/// `Gone` when memory runs out.
pub fn key(call: &NetCall<'_>) -> Result<(String, Zeroizing<Vec<u8>>), NetFault> {
    let (name, before) = match &call.header {
        NetHeader::Bearer => (AUTHORIZATION, BEARER),
        NetHeader::Named(name) => (name.as_ref(), &b""[..]),
    };
    let mut value = Zeroizing::new(Vec::new());
    value
        .try_reserve_exact(before.len() + call.key.len())
        .map_err(|_| NetFault::Gone)?;
    value.extend_from_slice(before);
    value.extend_from_slice(&call.key);
    Ok((name.to_owned(), value))
}
