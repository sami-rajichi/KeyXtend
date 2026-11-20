//! What a field's content may hold, checked by both sides after its length: a number within its range, a short ASCII
//! name, a host name, a path without a NUL and a name without control characters or hidden format marks. The tag, host,
//! host-list and cloud request rules are public, so the ports and the jobs refuse what the frames refuse.

use crate::error::IpcError;
use crate::limits::{HTTPS, MAX_HOST_BYTES, MAX_RECORD_BYTES, SAMPLE_BYTES};
use std::ops::RangeInclusive;

/// The marks a request's header or form field name may hold besides ASCII letters and digits.
const TOKEN_MARKS: &[u8] = b"-_";
/// The one byte a form value may hold besides visible ASCII.
const SPACE: u8 = b' ';
/// What ends an address's host and port: its path or its query.
const HOST_END: [char; 2] = ['/', '?'];
/// Parts a host from its port.
const PORT_MARK: char = ':';
/// Marks a user name or a fragment, which a service's address never holds.
const NOT_IN: [char; 2] = ['@', '#'];
/// The marks a link may hold besides ASCII letters and digits: RFC 3986's unreserved and reserved ones, and `%`.
const LINK_MARKS: &[u8] = b"-._~:/?#[]@!$&'()*+,;=%";
/// Starts a hex number, which a URL reader takes as part of an IPv4 address.
const HEX_MARK: &str = "0x";
/// The marks a model's id may hold besides a name's: a version's dot, a provider's slash and a tag's colon.
const MODEL_MARKS: &[u8] = b"./:";
/// Parts a model id's path segments, as in `vendor/model-v3`.
const SEGMENT_MARK: char = '/';
/// Path segments that would climb or stay in an address's path.
const DOT_SEGMENTS: [&str; 2] = [".", ".."];
/// The header a bearer key goes in.
pub const AUTHORIZATION: &str = "Authorization";
/// The header that names a body's type, a request's and a form part's.
pub const CONTENT_TYPE: &str = "Content-Type";
/// The headers every cloud request sends besides a row's: the bearer key's, the body's type and length, the host,
/// the user agent the transport names, and the connection header `WinHTTP` adds.
const SET_HEADERS: [&str; 6] = [
    AUTHORIZATION,
    CONTENT_TYPE,
    "Content-Length",
    "Host",
    "User-Agent",
    "Connection",
];

/// Format marks that reorder or hide text in a right-to-left panel: the Arabic letter mark, zero-width and direction
/// marks, embeddings and overrides, invisible operators, isolates and the byte-order mark.
const HIDDEN: &[RangeInclusive<char>] = &[
    '\u{061C}'..='\u{061C}',
    '\u{200B}'..='\u{200F}',
    '\u{202A}'..='\u{202E}',
    '\u{2060}'..='\u{2064}',
    '\u{2066}'..='\u{2069}',
    '\u{FEFF}'..='\u{FEFF}',
];

/// `n` when it is within `range`, else `BadField`.
pub(crate) fn within<T: PartialOrd>(n: T, range: &RangeInclusive<T>) -> Result<T, IpcError> {
    if range.contains(&n) {
        Ok(n)
    } else {
        Err(IpcError::BadField)
    }
}

/// Refuses anything but visible ASCII, for a language code, a runner, a card's id or a model's id.
///
/// # Errors
/// `BadField` for any other byte.
pub fn tag(text: &str) -> Result<(), IpcError> {
    visible(text.as_bytes())
}

/// Refuses a cloud key unless it is visible ASCII, so no request's head misreads it.
///
/// # Errors
/// `BadField` for any other byte.
pub fn key(bytes: &[u8]) -> Result<(), IpcError> {
    visible(bytes)
}

/// Refuses a request's header or form field name unless it is ASCII letters, digits and `TOKEN_MARKS`.
///
/// # Errors
/// `BadField` for any other byte.
pub fn token(text: &str) -> Result<(), IpcError> {
    ok_if(
        text.bytes()
            .all(|b| b.is_ascii_alphanumeric() || TOKEN_MARKS.contains(&b)),
    )
}

/// Refuses a key header's name unless it is a `token` naming none of `SET_HEADERS` but `AUTHORIZATION`, in any case,
/// so the key repeats no header the request sets.
///
/// # Errors
/// `BadField` for any other name.
pub fn key_header(name: &str) -> Result<(), IpcError> {
    token(name)?;
    let taken = |set: &&str| set.eq_ignore_ascii_case(name) && *set != AUTHORIZATION;
    ok_if(!SET_HEADERS.iter().any(taken))
}

/// Refuses a header name a request already sets, in any case: `key`, the key's own, or one of `SET_HEADERS`.
///
/// # Errors
/// `BadField` for such a name.
pub fn unset_header(name: &str, key: &str) -> Result<(), IpcError> {
    let taken = |set: &&str| set.eq_ignore_ascii_case(name);
    ok_if(!SET_HEADERS.iter().chain([&key]).any(taken))
}

/// Refuses a model's id unless it is `token` bytes and `MODEL_MARKS` in `/` segments, none empty, `.` or `..`, so it
/// breaks no body and climbs no address.
///
/// # Errors
/// `BadField` for any other id.
pub fn model(text: &str) -> Result<(), IpcError> {
    let plain =
        |b: u8| b.is_ascii_alphanumeric() || TOKEN_MARKS.contains(&b) || MODEL_MARKS.contains(&b);
    let segment = |s: &str| !s.is_empty() && !DOT_SEGMENTS.contains(&s);
    ok_if(text.bytes().all(plain) && text.split(SEGMENT_MARK).all(segment))
}

/// Refuses a form field's value unless it is visible ASCII and spaces, so it holds no line break.
///
/// # Errors
/// `BadField` for any other byte.
pub fn form_value(text: &str) -> Result<(), IpcError> {
    ok_if(text.bytes().all(|b| b.is_ascii_graphic() || b == SPACE))
}

/// Refuses a cloud piece of `bytes` unless it is whole samples, one at least, within `most`; it adds the piece to
/// `total`, which a run keeps within `MAX_RECORD_BYTES`.
///
/// # Errors
/// `BadField` for an empty piece, a part sample, a piece past `most` or a run past the longest dictation.
pub fn cloud_piece(bytes: usize, most: usize, total: &mut usize) -> Result<(), IpcError> {
    *total = total.saturating_add(bytes);
    let whole = bytes > 0 && bytes.is_multiple_of(SAMPLE_BYTES);
    ok_if(whole && bytes <= most && *total <= MAX_RECORD_BYTES)
}

/// True for a byte a link may hold: an ASCII letter or digit, or one of RFC 3986's marks.
#[must_use]
pub fn link_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || LINK_MARKS.contains(&b)
}

/// Refuses a service's address unless it is `HTTPS`, link bytes without `NOT_IN`, then a plain host (`listed`,
/// never `UNDER`, never a number) within `MAX_HOST_BYTES` and maybe `:` and a port from 1; its requests may reach only
/// that host.
///
/// # Errors
/// `BadField` for anything else.
pub fn https(text: &str) -> Result<(), IpcError> {
    let plain = text.starts_with(HTTPS) && !text.contains(NOT_IN);
    ok_if(text.bytes().all(link_byte))?;
    let (host, port) = host_port(text);
    let port_ok = port.is_none_or(|p| digits(p) && p.parse::<u16>().is_ok_and(|n| n > 0));
    let short = !host.starts_with(UNDER) && host.len() <= MAX_HOST_BYTES;
    ok_if(plain && port_ok && short && !numeric(host))?;
    listed(host)
}

/// True for a host whose last label is all digits or `HEX_MARK` and hex digits, which a URL reader takes as an IPv4
/// address, as `2130706433` or `0x7f.1`; refusing it fails closed.
fn numeric(host: &str) -> bool {
    let last = host.rsplit(UNDER).next().unwrap_or_default();
    let hex = last
        .strip_prefix(HEX_MARK)
        .is_some_and(|h| h.bytes().all(|b| b.is_ascii_hexdigit()));
    hex || digits(last)
}

/// True when `text` is all ASCII digits.
fn digits(text: &str) -> bool {
    text.bytes().all(|b| b.is_ascii_digit())
}

/// The host of an address `https` takes, in lower case: the only one its requests may reach.
#[must_use]
pub fn https_host(text: &str) -> &str {
    host_port(text).0
}

/// The scheme, host and maybe port of an address, up to its path or query; empty for a text that is no https one.
#[must_use]
pub fn https_origin(text: &str) -> &str {
    let Some(rest) = text.strip_prefix(HTTPS) else {
        return "";
    };
    let auth = rest.split(HOST_END).next().unwrap_or_default();
    &text[..HTTPS.len() + auth.len()]
}

/// The host and maybe the port of an address, between `HTTPS` and its path or query.
fn host_port(text: &str) -> (&str, Option<&str>) {
    let rest = text.strip_prefix(HTTPS).unwrap_or_default();
    let auth = rest.split(HOST_END).next().unwrap_or_default();
    auth.rsplit_once(PORT_MARK)
        .map_or((auth, None), |(host, port)| (host, Some(port)))
}

/// Refuses any byte but visible ASCII.
fn visible(bytes: &[u8]) -> Result<(), IpcError> {
    ok_if(bytes.iter().all(u8::is_ascii_graphic))
}

/// Refuses a host that is not lower-case letters, digits, dots and hyphens.
///
/// # Errors
/// `BadField` for any other byte.
pub fn host(text: &str) -> Result<(), IpcError> {
    let plain = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-';
    ok_if(text.bytes().all(plain))
}

/// Starts a listed host that takes each host under it, as `.hf.co` takes `us.aws.cdn.hf.co`.
pub const UNDER: char = '.';
/// The fewest names after `UNDER`, so a bare top-level domain such as `.co` never passes. A two-name public suffix such
/// as `.co.uk` still does, so the models table's test pins its `UNDER` entries.
pub const UNDER_NAMES: usize = 2;

/// Refuses a download's listed host unless it is a plain host, or `UNDER` and a domain of `UNDER_NAMES` names or more;
/// it knows no public-suffix list.
///
/// # Errors
/// `BadField` for an empty entry, a byte `host` refuses, an empty name, or too few names after `UNDER`.
pub fn listed(entry: &str) -> Result<(), IpcError> {
    host(entry)?;
    let (domain, least) = match entry.strip_prefix(UNDER) {
        Some(domain) => (domain, UNDER_NAMES),
        None => (entry, 1),
    };
    let names = domain.split(UNDER);
    ok_if(names.clone().all(|n| !n.is_empty()) && names.count() >= least)
}

/// True when the listed `entry` takes the plain `host`: the same host, or one under a good `UNDER` entry's domain.
#[must_use]
pub fn covers(entry: &str, host: &str) -> bool {
    let plain = !host.starts_with(UNDER) && listed(host).is_ok();
    let under = entry.starts_with(UNDER) && listed(entry).is_ok();
    plain && (entry == host || (under && host.len() > entry.len() && host.ends_with(entry)))
}

/// True when the listed `entry` is the plain `host` itself, never an `UNDER` entry: a download's first address must be
/// listed by name, while its redirects may be `covers`'.
#[must_use]
pub fn names(entry: &str, host: &str) -> bool {
    !entry.starts_with(UNDER) && covers(entry, host)
}

/// Refuses a path holding a NUL, which ends a path early for the OS.
pub(crate) fn path(text: &str) -> Result<(), IpcError> {
    ok_if(!text.contains('\0'))
}

/// Refuses a shown name holding a control character, such as a line break, or a `HIDDEN` mark.
pub(crate) fn name(text: &str) -> Result<(), IpcError> {
    ok_if(!text.chars().any(|c| c.is_control() || hidden(c)))
}

/// True for a mark of `HIDDEN`.
fn hidden(c: char) -> bool {
    HIDDEN.iter().any(|run| run.contains(&c))
}

/// `Ok` when `good`, else `BadField`.
pub(crate) fn ok_if(good: bool) -> Result<(), IpcError> {
    if good {
        Ok(())
    } else {
        Err(IpcError::BadField)
    }
}
