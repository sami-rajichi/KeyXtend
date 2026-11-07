//! The backup blob's bytes: a mark and a version, the first offer, the stop's flag and the hold, the padlock, the
//! memory checks, the places, then the marks left behind, every text after its length. A read checks what is left
//! before it slices, so random bytes give `Bad`, never a panic or a large allocation.

use std::path::PathBuf;

use kx_module_api::ports::{MAX_BLOB_BYTES, Padlock, Place, PlaceKind};
use kx_module_api::{Redacted, Zeroizing};

use super::{Checks, Kept, Site};
use crate::entry::Stamp;
use crate::store::{Bad, Count, FLAG, LEN, Reader, Sink, put_bytes, put_flag, put_len, put_text};

/// What every saved backup blob starts with.
pub(super) const MARK: [u8; 4] = *b"KXBK";
/// The format's version, the byte after the mark; a newer one is refused, never read.
const VERSION: u8 = 1;
/// The fewest bytes a place takes: its kind, its name's and path's lengths, its box length and its three flags.
const SITE_LEAST: usize = 1 + LEN + LEN + size_of::<u64>() + 3 * FLAG;
/// The bytes of a left mark.
const MARK_LEN: usize = size_of::<u64>();

/// `kept` as bytes, in one wiped buffer sized first.
#[must_use]
pub fn encode(kept: &Kept) -> Zeroizing<Vec<u8>> {
    let mut count = Count(0);
    write(&mut count, kept);
    let mut out = Zeroizing::new(Vec::with_capacity(count.0));
    write(&mut *out, kept);
    out
}

/// Writes the head, the flags, the padlock, the checks, the places and the left marks.
fn write(sink: &mut impl Sink, kept: &Kept) {
    sink.put(&MARK);
    sink.put(&[VERSION]);
    put_flag(sink, kept.offered);
    put_flag(sink, kept.behind);
    put_flag(sink, kept.held);
    put_flag(sink, kept.padlock.is_some());
    if let Some(padlock) = &kept.padlock {
        put_text(sink, padlock.recipient.expose());
        put_bytes(sink, padlock.locked.expose());
    }
    put_stamp(sink, kept.checks.due);
    sink.put(&[kept.checks.step, kept.checks.missed]);
    put_len(sink, kept.sites.len());
    kept.sites.iter().for_each(|site| write_site(sink, site));
    put_len(sink, kept.left.len());
    kept.left.iter().for_each(|m| sink.put(&m.to_le_bytes()));
}

/// Writes one place: its kind, name, path, last write, mark, box length and wait.
fn write_site(sink: &mut impl Sink, site: &Site) {
    sink.put(&[kind_tag(site.place.kind)]);
    put_text(sink, site.place.name.expose());
    put_text(sink, &site.place.path.expose().to_string_lossy());
    put_stamp(sink, site.written);
    put_flag(sink, site.mark.is_some());
    if let Some(mark) = site.mark {
        sink.put(&mark.to_le_bytes());
    }
    sink.put(&site.len.to_le_bytes());
    put_flag(sink, site.waiting);
}

/// Writes a time that may be missing.
fn put_stamp(sink: &mut impl Sink, at: Option<Stamp>) {
    put_flag(sink, at.is_some());
    if let Some(at) = at {
        sink.put(&at.unix_ms().to_le_bytes());
    }
}

/// What `bytes` keep.
///
/// # Errors
/// `Newer` for a newer version's blob, else `Damaged` for bytes that do not read.
pub fn decode(bytes: &[u8]) -> Result<Kept, Bad> {
    let mut r = Reader(bytes);
    r.head(&MARK, VERSION)?;
    let (offered, behind, held) = (r.flag()?, r.flag()?, r.flag()?);
    let padlock = r.flag()?.then(|| read_padlock(&mut r)).transpose()?;
    let due = read_stamp(&mut r)?;
    let checks = Checks {
        due,
        step: r.byte()?,
        missed: r.byte()?,
    };
    let count = r.count(MAX_BLOB_BYTES)?;
    let mut sites = Vec::with_capacity(count.min(r.0.len() / SITE_LEAST));
    for _ in 0..count {
        sites.push(read_site(&mut r)?);
    }
    let count = r.count(MAX_BLOB_BYTES)?;
    let mut left = Vec::with_capacity(count.min(r.0.len() / MARK_LEN));
    for _ in 0..count {
        left.push(r.u64()?);
    }
    r.end()?;
    Ok(Kept {
        sites,
        padlock,
        left,
        offered,
        behind,
        held,
        checks,
    })
}

/// Reads a padlock: its recipient, then its locked identity.
fn read_padlock(r: &mut Reader<'_>) -> Result<Padlock, Bad> {
    let recipient = r.text(MAX_BLOB_BYTES)?.to_owned();
    let locked = r.bytes(MAX_BLOB_BYTES)?.to_vec();
    Ok(Padlock {
        recipient: Redacted::new(recipient),
        locked: Redacted::new(locked),
    })
}

/// Reads one place.
fn read_site(r: &mut Reader<'_>) -> Result<Site, Bad> {
    let kind = tag_kind(r.byte()?).ok_or(Bad::Damaged)?;
    let name = r.text(MAX_BLOB_BYTES)?.to_owned();
    let path = PathBuf::from(r.text(MAX_BLOB_BYTES)?);
    let written = read_stamp(r)?;
    let mark = r.flag()?.then(|| r.u64()).transpose()?;
    Ok(Site {
        place: Place {
            kind,
            name: Redacted::new(name),
            path: Redacted::new(path),
        },
        written,
        mark,
        len: r.u64()?,
        waiting: r.flag()?,
    })
}

/// Reads a time that may be missing.
fn read_stamp(r: &mut Reader<'_>) -> Result<Option<Stamp>, Bad> {
    r.flag()?
        .then(|| r.i64().map(Stamp::from_unix_ms))
        .transpose()
}

/// A place kind's saved tag; saved blobs hold it, so it never changes.
const fn kind_tag(kind: PlaceKind) -> u8 {
    match kind {
        PlaceKind::Cloud => 0,
        PlaceKind::Removable => 1,
        PlaceKind::Drive => 2,
        PlaceKind::Folder => 3,
        PlaceKind::File => 4,
    }
}

/// The place kind of a saved tag.
fn tag_kind(tag: u8) -> Option<PlaceKind> {
    PlaceKind::ALL.into_iter().find(|&k| kind_tag(k) == tag)
}
