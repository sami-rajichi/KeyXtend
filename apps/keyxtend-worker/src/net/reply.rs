//! Reading a service's reply, an untrusted body: read within a cap, refused one byte past it and ended once its call
//! stops, its JSON `text` read straight into a wiped string, where its error's code sits, and the headers a 429 is
//! judged by. Every reader takes any bytes without a panic.

use kx_ipc::NetFault;
use kx_platform_net::Body;
use serde::Deserialize;
use serde::de::{self, Deserializer, IgnoredAny, Visitor};
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use zeroize::Zeroizing;

/// The most bytes one read of a body takes.
const READ: usize = 64 * 1024;
/// Nanoseconds in each unit a renewal's span may name, longest name first so `ms` is tried before `m`.
const UNITS: &[(&str, u128)] = &[
    ("ms", 1_000_000),
    ("h", 3_600_000_000_000),
    ("m", 60_000_000_000),
    ("s", 1_000_000_000),
];
/// Nanoseconds in a second.
const NS_PER_S: u128 = 1_000_000_000;
/// The most digits one number of a span may have, so its sum stays far within `u128`.
const MAX_DIGITS: usize = 20;
/// Parts a span number's whole part from its fraction.
const POINT: char = '.';
/// Base ten, for a span's fraction.
const TEN: u128 = 10;
/// The names an error's words sit under: `error` or its message, type and code, and a 422's `detail[]` with each item's
/// `msg`, `type` and `loc`, at the top as `DeepInfra` sends it or under `message`.
pub const SAYING: &[&str] = &["error", "message", "type", "code", "detail", "msg", "loc"];

/// `body`'s bytes, read into wiped chunks of `READ` bytes, each filled before the next, then joined once into a wiped
/// buffer sized once, so no growth leaves an unwiped copy and the room peaks near twice `cap`. `stop` is looked at
/// before each read.
///
/// # Errors
/// `Strange` once the body passes `cap`, read no further; `Gone` once `stop` is set; `Network`, `Timeout` or `Gone`
/// when a read fails.
pub fn drain(
    body: &mut dyn Body,
    cap: usize,
    stop: &AtomicBool,
) -> Result<Zeroizing<Vec<u8>>, NetFault> {
    let (mut filled, mut chunk, mut fill, mut got) = (Vec::new(), chunk()?, 0, 0usize);
    loop {
        if fill == chunk.len() {
            filled.push(std::mem::replace(&mut chunk, self::chunk()?));
            fill = 0;
        }
        if stop.load(Ordering::SeqCst) {
            return Err(NetFault::Gone);
        }
        let want = (chunk.len() - fill).min(cap - got + 1);
        let n = body
            .read(&mut chunk[fill..fill + want])
            .map_err(NetFault::from)?;
        if n == 0 {
            break;
        }
        let n = n.min(want);
        (fill, got) = (fill + n, got + n);
        if got > cap {
            return Err(NetFault::Strange);
        }
    }
    let mut all = Zeroizing::new(Vec::new());
    all.try_reserve_exact(got).map_err(|_| NetFault::Gone)?;
    for c in &filled {
        all.extend_from_slice(c);
    }
    all.extend_from_slice(&chunk[..fill]);
    Ok(all)
}

/// An empty wiped chunk of `READ` bytes.
fn chunk() -> Result<Zeroizing<Vec<u8>>, NetFault> {
    let mut chunk = Zeroizing::new(Vec::new());
    chunk.try_reserve_exact(READ).map_err(|_| NetFault::Gone)?;
    chunk.resize(READ, 0);
    Ok(chunk)
}

/// A JSON string read straight into a wiped buffer.
pub struct Wiped(pub Zeroizing<String>);

/// Reads a `Wiped`.
struct WipedVisitor;

impl Visitor<'_> for WipedVisitor {
    type Value = Wiped;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a string")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Wiped, E> {
        Ok(Wiped(Zeroizing::new(v.to_owned())))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Wiped, E> {
        Ok(Wiped(Zeroizing::new(v)))
    }
}

impl<'de> Deserialize<'de> for Wiped {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_string(WipedVisitor)
    }
}

/// A reply that holds its text; other fields are skipped unread.
#[derive(Deserialize)]
struct Said {
    text: Wiped,
}

/// What starts a JSON object.
const OBJECT: u8 = b'{';

/// The `text` a JSON reply object holds; none when the reply is not that JSON.
pub fn text(bytes: &[u8]) -> Option<Zeroizing<String>> {
    let said = opens_object(bytes).then(|| serde_json::from_slice::<Said>(bytes).ok());
    said.flatten().map(|s| s.text.0)
}

/// True when `bytes` start as a JSON object; serde would read a reply's struct from an array too.
pub fn opens_object(bytes: &[u8]) -> bool {
    bytes.trim_ascii_start().first() == Some(&OBJECT)
}

/// True when `bytes` is one JSON object, read through without keeping any of it.
pub fn object(bytes: &[u8]) -> bool {
    opens_object(bytes) && serde_json::from_slice::<IgnoredAny>(bytes).is_ok()
}

/// Where a JSON error reply keeps its code: the object at its top, and the names the code may sit under there.
pub struct Coded {
    /// The object's name.
    pub object: &'static str,
    /// The names its code may sit under.
    pub names: &'static [&'static str],
}

/// No code, for a reply whose table reads words alone.
pub const UNCODED: Coded = Coded {
    object: "",
    names: &[],
};

/// The upload form's errors (Groq's and `OpenAI`'s pages): `error.code` or `error.type`.
pub const ERROR: Coded = Coded {
    object: "error",
    names: &["code", "type"],
};

/// A `Retry-After` of whole seconds; none for a date or anything else.
pub fn seconds(text: &str) -> Option<u32> {
    let text = text.trim();
    let digits = !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit());
    digits.then(|| text.parse().ok()).flatten()
}

/// The whole seconds, rounded up, a span such as `2m59.56s` names in `h`, `m`, `s` and `ms`; none for anything else.
pub fn span_s(text: &str) -> Option<u64> {
    let (mut rest, mut ns) = (text, 0u128);
    if rest.is_empty() {
        return None;
    }
    while !rest.is_empty() {
        let end = rest.find(|c: char| !c.is_ascii_digit() && c != POINT)?;
        let (number, tail) = rest.split_at(end);
        let (unit, per) = UNITS.iter().find(|(u, _)| tail.starts_with(u))?;
        ns = ns.checked_add(part_ns(number, *per)?)?;
        rest = &tail[unit.len()..];
    }
    u64::try_from(ns.div_ceil(NS_PER_S)).ok()
}

/// Nanoseconds in `number` of a unit `per` nanoseconds long, its fraction rounded up.
fn part_ns(number: &str, per: u128) -> Option<u128> {
    let (whole, fraction) = number.split_once(POINT).unwrap_or((number, ""));
    let plain = |d: &str| d.len() <= MAX_DIGITS && d.bytes().all(|b| b.is_ascii_digit());
    if (whole.is_empty() && fraction.is_empty()) || !plain(whole) || !plain(fraction) {
        return None;
    }
    let value = |d: &str| {
        if d.is_empty() {
            Some(0)
        } else {
            d.parse::<u128>().ok()
        }
    };
    let scale = TEN.checked_pow(u32::try_from(fraction.len()).ok()?)?;
    let part = value(fraction)?.checked_mul(per)?.div_ceil(scale);
    value(whole)?.checked_mul(per)?.checked_add(part)
}
