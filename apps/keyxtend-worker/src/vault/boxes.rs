//! The backup box (ADR-0038 decision 6): a padlock is a new X25519 identity locked by scrypt under the password, and
//! its open padlock, the recipient. A box is the mark and version, the locked identity, then all its named parts
//! sealed to the recipient as one `age` file, so sealing needs no password and opening needs it once.

mod cipher;
mod layout;

use age::x25519;
use kx_ipc::{BOX_PARTS, BoxPart, Fault, MAX_BOX_WORK, MIN_BOX_WORK, Wiped};
use std::borrow::Cow;
use std::ops::Range;
use zeroize::Zeroizing;

/// A padlock: what seals new boxes, and the identity that opens them, locked.
#[derive(Debug, PartialEq, Eq)]
pub struct Padlock {
    /// The open padlock: the identity's `age` X25519 recipient.
    pub recipient: String,
    /// The identity, locked by scrypt under the password.
    pub locked: Vec<u8>,
}

/// What an opened box holds.
#[derive(Debug)]
pub struct Opened {
    /// The box's own padlock.
    pub padlock: Padlock,
    /// Its parts, in the box's order.
    pub parts: Vec<BoxPart<'static>>,
}

/// Makes a padlock whose identity `password` locks at scrypt work factor `work` (log N).
///
/// # Errors
/// `TooLarge` outside `MIN_BOX_WORK` to `MAX_BOX_WORK`; `Internal` when `age` fails.
pub fn padlock(password: &str, work: u8) -> Result<Padlock, Fault> {
    if !(MIN_BOX_WORK..=MAX_BOX_WORK).contains(&work) {
        return Err(Fault::TooLarge);
    }
    let identity = x25519::Identity::generate();
    let locked = cipher::lock(&identity, password, work)?;
    let recipient = identity.to_public().to_string();
    Ok(Padlock { recipient, locked })
}

/// Seals `parts` into a box under the padlock `recipient` and its `locked` identity; it needs no password.
///
/// # Errors
/// `Damaged` for a padlock that will not read; `Internal` for no part, which the frames refuse, or when `age` fails.
pub fn seal(recipient: &str, locked: &[u8], parts: &[BoxPart<'_>]) -> Result<Vec<u8>, Fault> {
    let recipient: x25519::Recipient = recipient.parse().map_err(|_| Fault::Damaged)?;
    if !cipher::is_locked(locked) {
        return Err(Fault::Damaged);
    }
    if parts.is_empty() {
        return Err(Fault::Internal);
    }
    let mut out = layout::begin(locked)?;
    cipher::seal_parts(&recipient, parts, &mut out)?;
    Ok(out)
}

/// Opens a box's `bytes` with `password`: its layout first, then its identity, then each part.
///
/// # Errors
/// `Foreign`, `Damaged` or `Unsupported` for a box that is not one, is cut or forged, or is newer, before any key is
/// derived when its layout shows it; `WrongPassword`; `Unsupported` for an unknown part; `Damaged` for a repeated one
/// or none.
pub fn open(bytes: &[u8], password: &str) -> Result<Opened, Fault> {
    let layout = layout::read(bytes)?;
    let identity = cipher::unlock(layout.locked, password)?;
    let parts = open_parts(&identity, layout.sealed)?;
    let padlock = Padlock {
        recipient: identity.to_public().to_string(),
        locked: layout.locked.to_vec(),
    };
    Ok(Opened { padlock, parts })
}

/// True when `password` opens the `locked` identity.
///
/// # Errors
/// `Damaged` when the identity will not read or asks for a work factor past `MAX_BOX_WORK`.
pub fn check(locked: &[u8], password: &str) -> Result<bool, Fault> {
    match cipher::unlock(locked, password) {
        Ok(_) => Ok(true),
        Err(Fault::WrongPassword) => Ok(false),
        Err(fault) => Err(fault),
    }
}

/// Opens the parts `sealed` holds with `identity`, naming each as the walk reaches it, so an unknown or repeated
/// name stops a long list there; a box with none is refused.
fn open_parts(identity: &x25519::Identity, sealed: &[u8]) -> Result<Vec<BoxPart<'static>>, Fault> {
    let plain = cipher::open_parts(identity, sealed)?;
    let mut named: Vec<(&'static str, Range<usize>)> = Vec::new();
    for part in layout::parts(&plain) {
        let (raw, range) = part?;
        let name = BOX_PARTS.iter().find(|n| n.as_bytes() == raw);
        let name = name.ok_or(Fault::Unsupported)?;
        if named.iter().any(|(n, _)| n == name) {
            return Err(Fault::Damaged);
        }
        named.push((name, range));
    }
    if named.is_empty() {
        return Err(Fault::Damaged);
    }
    Ok(cut(plain, &named))
}

/// Cuts `plain` into the `named` parts from the last, so the first keeps its buffer and no copy of it is made.
fn cut(
    mut plain: Zeroizing<Vec<u8>>,
    named: &[(&'static str, Range<usize>)],
) -> Vec<BoxPart<'static>> {
    let mut parts = Vec::with_capacity(named.len());
    for (at, (name, range)) in named.iter().enumerate().rev() {
        plain.truncate(range.end);
        let bytes = if at == 0 {
            plain.drain(..range.start);
            Zeroizing::new(std::mem::take(&mut *plain))
        } else {
            Zeroizing::new(plain.split_off(range.start))
        };
        parts.push(BoxPart {
            name: Cow::Borrowed(*name),
            bytes: Wiped::Kept(bytes),
        });
    }
    parts.reverse();
    parts
}

#[cfg(test)]
mod tests;
