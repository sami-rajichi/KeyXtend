//! A folder's list in one versioned byte form: the version byte, the round, the total length and the counter,
//! then the count and each entry's record. The store splits it into the head blob `list` and, past the blob size,
//! the blobs `list-<round>-<n>`; the rounds take turns, so a save never writes over the blobs the head names.

use std::mem::size_of;

use kx_module_api::Redacted;
use kx_module_api::ports::{AppRef, BlobName, ClipKind, SealedError};

use super::codec::{Bad, Reader, Writer};
use crate::entry::{Entry, Place, Stamp};

/// The list form's version, its first byte.
const VERSION: u8 = 1;
/// How many rounds the extra blobs take turns in.
const ROUNDS: u8 = 2;
/// The head blob's name, which also starts every extra blob's name.
const HEAD: &str = "list";
/// What stands between the parts of an extra blob's name.
const SEP: char = '-';
/// The bytes the head blob always holds: the version, the round and the total length.
pub(crate) const HEAD_LEN: usize = 2 * size_of::<u8>() + size_of::<u64>();

/// The text kind's tag.
const TEXT: u8 = 1;
/// The link kind's tag.
const LINK: u8 = 2;
/// The image kind's tag.
const IMAGE: u8 = 3;
/// The file kind's tag.
const FILE: u8 = 4;
/// The media kind's tag.
const MEDIA: u8 = 5;

/// A list as read: its round, the counter and the entries of its folder.
#[derive(Debug)]
pub(crate) struct List {
    /// The round of its extra blobs.
    pub(crate) round: u8,
    /// The history's counter when it was saved.
    pub(crate) next: u64,
    /// The entries, as saved.
    pub(crate) entries: Vec<Entry>,
}

/// The count and records of `entries`; the store compares them to know when a list changed.
pub(crate) fn records(entries: &[&Entry]) -> Vec<u8> {
    let mut w = Writer::default();
    w.count(entries.len());
    for entry in entries {
        record(&mut w, entry);
    }
    w.done()
}

/// The whole list of round `round` with the counter `next` and `records`.
pub(crate) fn frame(round: u8, next: u64, records: &[u8]) -> Vec<u8> {
    let total = HEAD_LEN + size_of::<u64>() + records.len();
    let mut w = Writer::default();
    w.u8(VERSION);
    w.u8(round);
    w.count(total);
    w.u64(next);
    let mut bytes = w.done();
    bytes.extend_from_slice(records);
    bytes
}

/// The round and the total length a head blob starts with.
pub(crate) fn head(bytes: &[u8]) -> Result<(u8, usize), Bad> {
    let mut r = Reader::new(bytes);
    if r.u8()? != VERSION {
        return Err(Bad);
    }
    let round = r.u8()?;
    let total = usize::try_from(r.u64()?).map_err(|_| Bad)?;
    if round < ROUNDS {
        Ok((round, total))
    } else {
        Err(Bad)
    }
}

/// The list in `bytes`, its entries kept in `place`.
pub(crate) fn decode(bytes: &[u8], place: Place) -> Result<List, Bad> {
    let (round, total) = head(bytes)?;
    if total != bytes.len() {
        return Err(Bad);
    }
    let mut r = Reader::new(bytes);
    r.take(HEAD_LEN)?;
    let next = r.u64()?;
    let n = r.count()?;
    let entries = (0..n)
        .map(|_| entry(&mut r, place))
        .collect::<Result<_, _>>()?;
    if r.done() {
        Ok(List {
            round,
            next,
            entries,
        })
    } else {
        Err(Bad)
    }
}

/// The round after `round`.
pub(crate) fn turn(round: u8) -> u8 {
    (round + 1) % ROUNDS
}

/// The head blob's name.
pub(crate) fn head_name() -> Result<BlobName, SealedError> {
    BlobName::new(HEAD)
}

/// The name of extra blob `n` of round `round`, from 1.
pub(crate) fn chunk_name(round: u8, n: usize) -> Result<BlobName, SealedError> {
    BlobName::new(&format!("{HEAD}{SEP}{round}{SEP}{n}"))
}

/// True when `name` is the head blob or an extra blob.
pub(crate) fn is_list(name: &BlobName) -> bool {
    let name = name.as_str();
    name == HEAD || name.strip_prefix(HEAD).is_some_and(|r| r.starts_with(SEP))
}

/// Writes one entry's record; the folder gives its place.
fn record(w: &mut Writer, e: &Entry) {
    w.u64(e.id);
    w.u64(e.seq);
    w.u8(kind_tag(e.kind));
    w.i64(e.at.unix_ms);
    w.i16(e.at.offset_min);
    w.flag(e.app.is_some());
    if let Some(app) = &e.app {
        w.text(&app.id);
        w.text(&app.name);
    }
    w.u64(e.stored);
    w.u64(e.shown);
    w.text(e.title.expose());
    w.text(e.search.expose());
    w.u64(e.hash);
    w.u16(e.parts);
    w.flag(e.thumb);
    w.paths(e.files.expose());
}

/// Reads one entry's record, in `place`.
fn entry(r: &mut Reader<'_>, place: Place) -> Result<Entry, Bad> {
    let id = r.u64()?;
    let seq = r.u64()?;
    let kind = kind_of(r.u8()?)?;
    let unix_ms = r.i64()?;
    let offset_min = r.i16()?;
    let app = if r.flag()? {
        let id = r.text()?.to_owned();
        let name = r.text()?.to_owned();
        Some(AppRef { id, name })
    } else {
        None
    };
    let (stored, shown) = (r.u64()?, r.u64()?);
    let title = Redacted::new(r.text()?.to_owned());
    let search = Redacted::new(r.text()?.to_owned());
    let (hash, parts, thumb) = (r.u64()?, r.u16()?, r.flag()?);
    let files = Redacted::new(r.paths()?);
    let at = Stamp {
        unix_ms,
        offset_min,
    };
    Ok(Entry {
        id,
        seq,
        kind,
        place,
        at,
        app,
        stored,
        shown,
        title,
        search,
        hash,
        parts,
        thumb,
        files,
    })
}

/// The kind's tag.
fn kind_tag(kind: ClipKind) -> u8 {
    match kind {
        ClipKind::Text => TEXT,
        ClipKind::Link => LINK,
        ClipKind::Image => IMAGE,
        ClipKind::File => FILE,
        ClipKind::Media => MEDIA,
    }
}

/// The kind a tag names.
fn kind_of(tag: u8) -> Result<ClipKind, Bad> {
    match tag {
        TEXT => Ok(ClipKind::Text),
        LINK => Ok(ClipKind::Link),
        IMAGE => Ok(ClipKind::Image),
        FILE => Ok(ClipKind::File),
        MEDIA => Ok(ClipKind::Media),
        _ => Err(Bad),
    }
}

#[cfg(test)]
mod tests;
