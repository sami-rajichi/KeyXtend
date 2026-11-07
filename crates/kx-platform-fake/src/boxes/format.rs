//! The fake box's bytes: the box's mark and version, then a fake tag, the seal's number, the padlock and the named
//! parts, each length-prefixed, and a checksum, so a damaged, cut or foreign box will not open. A padlock's locked
//! identity is its number and a hash of the password, and its recipient names the same number.

use kx_module_api::ports::{
    BOX_MARK, BOX_PARTS, BOX_VERSION, BoxError, BoxPart, MAX_BOX_BYTES, Opened, Padlock,
};
use kx_module_api::{Redacted, Secret, Zeroizing};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::atomic::{self, AtomicU64};

/// What follows the mark and version in a fake box, so a box from the real worker is foreign.
pub(super) const TAG: &[u8] = b"fake";
/// What starts a fake padlock's recipient, before its number.
const RECIPIENT: &str = "fake-recipient-";
/// The bytes of a length or a count.
const LEN: usize = size_of::<u32>();
/// The bytes of a number, a password's hash or the checksum.
const WORD: usize = size_of::<u64>();
/// The bytes of the mark and version.
const HEAD: usize = BOX_MARK.len() + 1;

/// The next padlock's number, so padlocks differ across every fake in the test run.
static PADLOCKS: AtomicU64 = AtomicU64::new(0);
/// The next seal's number, so two seals of the same parts differ, as `age`'s do.
static SEALS: AtomicU64 = AtomicU64::new(0);

/// Takes the next number from `counter`.
fn next(counter: &AtomicU64) -> u64 {
    counter.fetch_add(1, atomic::Ordering::Relaxed)
}

/// A hash of `value`, the same within one build.
fn hash(value: impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// The hash that stands for padlock `id`'s identity locked by `password`.
fn key(id: u64, password: &Secret) -> u64 {
    hash((id, password.expose().as_str()))
}

/// The recipient of padlock `id`'s identity.
fn recipient(id: u64) -> String {
    format!("{RECIPIENT}{id}")
}

/// A new padlock, unique, locked by `password`.
pub(super) fn padlock(password: &Secret) -> Padlock {
    let id = next(&PADLOCKS);
    let locked = [id.to_le_bytes(), key(id, password).to_le_bytes()].concat();
    Padlock {
        recipient: Redacted::new(recipient(id)),
        locked: Redacted::new(locked),
    }
}

/// The number and password hash a `locked` identity holds, `Damaged` when it will not read.
fn unlock(locked: &[u8]) -> Result<(u64, u64), BoxError> {
    let mut reader = Reader(locked);
    let (id, sum) = (reader.word()?, reader.word()?);
    reader.end()?;
    Ok((id, sum))
}

/// True when `password` locked `padlock`.
pub(super) fn check(padlock: &Padlock, password: &Secret) -> Result<bool, BoxError> {
    let (id, sum) = unlock(padlock.locked.expose())?;
    Ok(sum == key(id, password))
}

/// The bytes of a box holding `parts` sealed to `padlock`, checked in the platform's order: `Failed` for no part or an
/// unknown or repeated one, which the ask's frame refuses, `TooLarge` for parts past `MAX_BOX_BYTES`, and `Damaged`
/// for a padlock that will not read, as the worker answers.
pub(super) fn seal(padlock: &Padlock, parts: &[BoxPart]) -> Result<Vec<u8>, BoxError> {
    let mut seen = HashSet::new();
    let known = |p: &BoxPart| BOX_PARTS.contains(&p.name.as_str());
    let named = parts
        .iter()
        .all(|p| known(p) && seen.insert(p.name.as_str()));
    if !named || parts.is_empty() {
        return Err(BoxError::Failed);
    }
    let size = |p: &BoxPart| p.bytes.expose().len();
    if parts.iter().map(size).fold(0, usize::saturating_add) > MAX_BOX_BYTES {
        return Err(BoxError::TooLarge);
    }
    let number = padlock.recipient.expose().strip_prefix(RECIPIENT);
    if number.is_none_or(|n| n.parse::<u64>().is_err()) {
        return Err(BoxError::Damaged);
    }
    unlock(padlock.locked.expose())?;
    encode(padlock, parts)
}

/// The bytes of a box holding `parts` sealed to `padlock`, unchecked, so a test can forge one.
pub(super) fn encode(padlock: &Padlock, parts: &[BoxPart]) -> Result<Vec<u8>, BoxError> {
    let chunk = |n: usize| LEN + n;
    let sized = parts
        .iter()
        .map(|p| chunk(p.name.len()) + chunk(p.bytes.expose().len()));
    let (recipient, locked) = (padlock.recipient.expose(), padlock.locked.expose());
    let fixed = HEAD + TAG.len() + WORD + chunk(recipient.len()) + chunk(locked.len()) + LEN;
    if fixed + sized.sum::<usize>() + WORD > MAX_BOX_BYTES {
        return Err(BoxError::TooLarge);
    }
    let mut body = [TAG, &next(&SEALS).to_le_bytes()[..]].concat();
    put(&mut body, recipient.as_bytes())?;
    put(&mut body, locked)?;
    put_len(&mut body, parts.len())?;
    for part in parts {
        put(&mut body, part.name.as_bytes())?;
        put(&mut body, part.bytes.expose())?;
    }
    let sum = hash(&body).to_le_bytes();
    Ok([&BOX_MARK[..], &[BOX_VERSION], &body, &sum].concat())
}

/// Opens a box's `bytes` with `password`; no part, or parts sealed to another recipient than the identity's, are
/// `Damaged`, as the worker answers.
pub(super) fn open(bytes: &[u8], password: &Secret) -> Result<Opened, BoxError> {
    let mut body = Reader(summed(versioned(bytes)?)?);
    if body.take(TAG.len())? != TAG {
        return Err(BoxError::Damaged);
    }
    body.word()?;
    let sealed_to = text(body.chunk()?)?;
    let locked = body.chunk()?.to_vec();
    let (id, sum) = unlock(&locked)?;
    if sum != key(id, password) {
        return Err(BoxError::WrongPassword);
    }
    let parts = body.parts()?;
    body.end()?;
    let own = recipient(id);
    if parts.is_empty() || sealed_to != own {
        return Err(BoxError::Damaged);
    }
    let padlock = Padlock {
        recipient: Redacted::new(own),
        locked: Redacted::new(locked),
    };
    Ok(Opened { padlock, parts })
}

/// What follows the mark and version of a box of this version.
fn versioned(bytes: &[u8]) -> Result<&[u8], BoxError> {
    if bytes.len() > MAX_BOX_BYTES {
        return Err(BoxError::TooLarge);
    }
    let mut head = Reader(bytes);
    if head.take(BOX_MARK.len())? != BOX_MARK {
        return Err(BoxError::Damaged);
    }
    match head.byte()?.cmp(&BOX_VERSION) {
        Ordering::Greater => Err(BoxError::Unsupported),
        Ordering::Less => Err(BoxError::Damaged),
        Ordering::Equal => Ok(head.0),
    }
}

/// The body before the checksum at the end of `rest`, when the checksum holds.
fn summed(rest: &[u8]) -> Result<&[u8], BoxError> {
    let at = rest.len().checked_sub(WORD).ok_or(BoxError::Damaged)?;
    let (body, sum) = rest.split_at(at);
    let held = (sum == hash(body).to_le_bytes()).then_some(body);
    held.ok_or(BoxError::Damaged)
}

/// Adds `bytes` to `out` after their length.
fn put(out: &mut Vec<u8>, bytes: &[u8]) -> Result<(), BoxError> {
    put_len(out, bytes.len())?;
    out.extend_from_slice(bytes);
    Ok(())
}

/// Adds the length or count `n` to `out`.
fn put_len(out: &mut Vec<u8>, n: usize) -> Result<(), BoxError> {
    let n = u32::try_from(n).map_err(|_| BoxError::TooLarge)?;
    out.extend_from_slice(&n.to_le_bytes());
    Ok(())
}

/// UTF-8 `bytes` as text.
fn text(bytes: &[u8]) -> Result<String, BoxError> {
    String::from_utf8(bytes.to_vec()).map_err(|_| BoxError::Damaged)
}

/// Reads a box's bytes from the front; anything cut short is `Damaged`.
struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    /// The next `n` bytes.
    fn take(&mut self, n: usize) -> Result<&'a [u8], BoxError> {
        let (taken, rest) = self.0.split_at_checked(n).ok_or(BoxError::Damaged)?;
        self.0 = rest;
        Ok(taken)
    }

    /// The next byte.
    fn byte(&mut self) -> Result<u8, BoxError> {
        let taken = self.take(1)?;
        taken.first().copied().ok_or(BoxError::Damaged)
    }

    /// The next length or count.
    fn size(&mut self) -> Result<usize, BoxError> {
        let bytes = self.take(LEN)?.try_into().map_err(|_| BoxError::Damaged)?;
        usize::try_from(u32::from_le_bytes(bytes)).map_err(|_| BoxError::Damaged)
    }

    /// The next number.
    fn word(&mut self) -> Result<u64, BoxError> {
        let bytes = self.take(WORD)?.try_into().map_err(|_| BoxError::Damaged)?;
        Ok(u64::from_le_bytes(bytes))
    }

    /// The next length-prefixed bytes.
    fn chunk(&mut self) -> Result<&'a [u8], BoxError> {
        let n = self.size()?;
        self.take(n)
    }

    /// The parts after their count, as the worker opens them: an unknown name is `Unsupported` and a repeated one
    /// `Damaged`; each takes bytes, so a false count runs out at once.
    fn parts(&mut self) -> Result<Vec<BoxPart>, BoxError> {
        let count = self.size()?;
        let mut parts: Vec<BoxPart> = Vec::new();
        for _ in 0..count {
            let raw = self.chunk()?;
            let name = BOX_PARTS.iter().find(|n| n.as_bytes() == raw);
            let name = name.ok_or(BoxError::Unsupported)?;
            if parts.iter().any(|p| p.name == *name) {
                return Err(BoxError::Damaged);
            }
            let bytes = Redacted::new(Zeroizing::new(self.chunk()?.to_vec()));
            let name = (*name).to_owned();
            parts.push(BoxPart { name, bytes });
        }
        Ok(parts)
    }

    /// Nothing is left.
    fn end(&self) -> Result<(), BoxError> {
        self.0.is_empty().then_some(()).ok_or(BoxError::Damaged)
    }
}
