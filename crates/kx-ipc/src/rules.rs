//! What a field's content may hold, checked by both sides after its length: a number within its range, a short ASCII
//! name, a host name, a path without a NUL and a name without control characters or hidden format marks. The tag, host
//! and host-list rules are public, so the ports and the fetch refuse what the frames refuse.

use crate::error::IpcError;
use std::ops::RangeInclusive;

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

/// Refuses anything but visible ASCII, for a language code, a runner or a card's id.
///
/// # Errors
/// `BadField` for any other byte.
pub fn tag(text: &str) -> Result<(), IpcError> {
    ok_if(text.bytes().all(|b| b.is_ascii_graphic()))
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
