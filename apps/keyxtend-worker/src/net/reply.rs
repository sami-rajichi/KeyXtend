//! Reading a service's reply, an untrusted body: read within a cap, refused one byte past it and ended once its call
//! stops, its JSON `text` read straight into a wiped string, its error's exact code or words, and the headers a 429 is
//! judged by. Every reader takes any bytes without a panic.

use kx_ipc::NetFault;
use kx_platform_net::Body;
use serde::Deserialize;
use serde::de::{self, Deserializer, Visitor};
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
/// The names an error's words sit under.
const SAYING: &[&str] = &["error", "message", "type", "code", "detail", "msg", "loc"];
/// How many levels of objects and lists the words are looked for in: a list of `loc` inside a `detail` item, inside
/// `detail`, inside `message`, inside the reply.
const SAYING_DEPTH: usize = 5;

/// `body`'s bytes, read into wiped chunks of `READ` bytes, each filled before the next, and joined once, so no growth
/// leaves an unwiped copy and the room taken stays within `cap` and one chunk; `stop` is looked at before each read.
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

/// True when `bytes` is one JSON object.
pub fn object(bytes: &[u8]) -> bool {
    serde_json::from_slice::<serde_json::Map<String, serde_json::Value>>(bytes).is_ok()
}

/// Where a JSON error reply keeps its code: the object at its top, and the names the code may sit under there.
pub struct Coded {
    /// The object's name.
    pub object: &'static str,
    /// The names its code may sit under.
    pub names: &'static [&'static str],
}

/// The upload form's errors (Groq's and `OpenAI`'s pages): `error.code` or `error.type`.
pub const ERROR: Coded = Coded {
    object: "error",
    names: &["code", "type"],
};

/// True when a JSON error reply's code, where `at` says, is exactly one of `codes`, in any case; never for another
/// body.
pub fn coded(bytes: &[u8], at: &Coded, codes: &[&str]) -> bool {
    let reply = serde_json::from_slice::<serde_json::Value>(bytes).unwrap_or_default();
    let is = |v: &serde_json::Value| {
        v.as_str()
            .is_some_and(|s| codes.iter().any(|c| c.eq_ignore_ascii_case(s)))
    };
    let object = reply.get(at.object);
    at.names
        .iter()
        .any(|name| object.and_then(|o| o.get(name)).is_some_and(is))
}

/// True when a JSON error reply's words hold one of `words`, in any case; never for another body. The words are the
/// texts under `SAYING`'s names, within `SAYING_DEPTH`: `error` or its message, type and code, and a 422's `detail[]`
/// with each item's `msg`, `type` and `loc`, at the top as `DeepInfra` sends it or under `message`.
pub fn says(bytes: &[u8], words: &[&str]) -> bool {
    says_in(bytes, SAYING, words)
}

/// True when the texts under `names` in a JSON error reply, within `SAYING_DEPTH`, hold one of `words`, in any case;
/// never for another body.
pub fn says_in(bytes: &[u8], names: &[&str], words: &[&str]) -> bool {
    let reply = serde_json::from_slice::<serde_json::Value>(bytes).unwrap_or_default();
    let mut said = String::new();
    if reply.is_object() {
        gather(&reply, names, SAYING_DEPTH, &mut said);
    }
    let said = said.to_ascii_lowercase();
    words.iter().any(|w| said.contains(w))
}

/// Adds the texts of `value` under `names` to `said`, `depth` levels down at most.
fn gather(value: &serde_json::Value, names: &[&str], depth: usize, said: &mut String) {
    let deeper = depth.checked_sub(1);
    match (value, deeper) {
        (serde_json::Value::String(text), _) => {
            said.push(' ');
            said.push_str(text);
        }
        (serde_json::Value::Array(items), Some(d)) => {
            for item in items {
                gather(item, names, d, said);
            }
        }
        (serde_json::Value::Object(map), Some(d)) => {
            for item in names.iter().filter_map(|name| map.get(*name)) {
                gather(item, names, d, said);
            }
        }
        _ => {}
    }
}

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
