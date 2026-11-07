//! The notebook's bytes: a mark and a version, then the entries, each with its id, kind, times, name and fields, every
//! text after its length. A read checks what is left before it slices and takes the format's caps, never the
//! settings, so lowering a limit loses nothing; random bytes give `Bad`, never a panic or a large allocation.

use std::collections::BTreeSet;

use kx_module_api::ports::{Label, MAX_FIELD_CHARS, MAX_VAULT_ENTRIES, MAX_VAULT_FIELDS};
use kx_module_api::{Redacted, Zeroizing};

pub use super::bytes::Bad;
use super::bytes::{Count, FLAG, LEN, Reader, Sink, put_flag, put_len, put_text};
use crate::entry::text::{blank, held};
use crate::entry::{Draft, Entry, Field, Stamp};
use crate::kinds::tables;

/// What every saved notebook starts with.
pub(super) const MARK: [u8; 4] = *b"KXQF";
/// The format's version, the byte after the mark; a newer one is refused, never read.
const VERSION: u8 = 1;
/// The most bytes a text takes: the worker's most characters, each at its widest in UTF-8.
const MAX_TEXT_BYTES: usize = MAX_FIELD_CHARS * char::MAX_LEN_UTF8;
/// The bytes the mark, the version and the entry count take.
pub const HEAD_LEN: usize = MARK.len() + size_of_val(&VERSION) + LEN;
/// The fewest bytes an entry takes: its id, kind length, change time, use flag, name length and field count.
const ENTRY_LEAST: usize = size_of::<u64>() + LEN + size_of::<i64>() + FLAG + LEN + LEN;
/// The tag of a table label, then of an own label.
const TABLE: u8 = 0;
const OWN: u8 = 1;

/// `entries` as bytes, in one buffer sized first, so no copy of a value is left unwiped.
#[must_use]
pub fn encode(entries: &[Entry]) -> Zeroizing<Vec<u8>> {
    let mut count = Count(0);
    write(&mut count, entries);
    let mut out = Zeroizing::new(Vec::with_capacity(count.0));
    write(&mut *out, entries);
    out
}

/// The bytes `entry` takes in a saved notebook.
#[must_use]
pub fn entry_len(entry: &Entry) -> usize {
    let mut count = Count(0);
    write_entry(&mut count, entry);
    count.0
}

/// Writes the head, then each entry.
fn write(sink: &mut impl Sink, entries: &[Entry]) {
    sink.put(&MARK);
    sink.put(&[VERSION]);
    put_len(sink, entries.len());
    for entry in entries {
        write_entry(sink, entry);
    }
}

/// Writes one entry: its id, kind, change time, use, name and fields.
fn write_entry(sink: &mut impl Sink, entry: &Entry) {
    sink.put(&entry.id().to_le_bytes());
    let kind = tables().kind(entry.kind()).map_or("", |r| r.id.as_str());
    put_text(sink, kind);
    sink.put(&entry.changed().unix_ms().to_le_bytes());
    put_flag(sink, entry.used().is_some());
    if let Some(used) = entry.used() {
        sink.put(&used.unix_ms().to_le_bytes());
    }
    put_text(sink, entry.name().expose());
    put_len(sink, entry.fields().len());
    for field in entry.fields() {
        match &field.label {
            Label::Table(id) => put_tagged(sink, TABLE, id),
            Label::Own(own) => put_tagged(sink, OWN, own.expose()),
        }
        put_flag(sink, field.hidden);
        put_text(sink, field.value.expose());
    }
}

/// Writes `tag`, then `text`.
fn put_tagged(sink: &mut impl Sink, tag: u8, text: &str) {
    sink.put(&[tag]);
    put_text(sink, text);
}

/// The entries `bytes` hold, each id once and every text within the rules.
///
/// # Errors
/// `Newer` for a newer version's notebook, else `Damaged` for bytes that do not read.
pub fn decode(bytes: &[u8]) -> Result<Vec<Entry>, Bad> {
    let mut r = Reader(bytes);
    r.head(&MARK, VERSION)?;
    let count = r.count(MAX_VAULT_ENTRIES)?;
    let room = count.min(r.0.len() / ENTRY_LEAST);
    let (mut entries, mut ids) = (Vec::with_capacity(room), BTreeSet::new());
    for _ in 0..count {
        let entry = read_entry(&mut r)?;
        if !ids.insert(entry.id()) {
            return Err(Bad::Damaged);
        }
        entries.push(entry);
    }
    r.end().map(|()| entries)
}

/// Reads one entry.
fn read_entry(r: &mut Reader<'_>) -> Result<Entry, Bad> {
    let id = r.u64()?;
    let kind = tables()
        .kind_by_id(r.text(MAX_TEXT_BYTES)?)
        .ok_or(Bad::Damaged)?
        .kind;
    let changed = Stamp::from_unix_ms(r.i64()?);
    let used = r
        .flag()?
        .then(|| r.i64().map(Stamp::from_unix_ms))
        .transpose()?;
    let name = Redacted::new(line(r)?.to_owned());
    let count = r.count(MAX_VAULT_FIELDS)?;
    let mut fields = Vec::with_capacity(count);
    for _ in 0..count {
        fields.push(read_field(r)?);
    }
    Ok(Entry::restored(
        id,
        Draft { name, kind, fields },
        changed,
        used,
    ))
}

/// Reads one field: a known table label or an own one not blank, the hidden flag, and a value that is not empty.
fn read_field(r: &mut Reader<'_>) -> Result<Field, Bad> {
    let label = match r.byte()? {
        TABLE => {
            let id = r.text(MAX_TEXT_BYTES)?;
            tables().label(id).ok_or(Bad::Damaged)?;
            Label::Table(id.to_owned())
        }
        OWN => {
            let own = line(r)?;
            if blank(own) {
                return Err(Bad::Damaged);
            }
            Label::Own(Redacted::new(own.to_owned()))
        }
        _ => return Err(Bad::Damaged),
    };
    let hidden = r.flag()?;
    let value = line(r)?;
    if value.is_empty() {
        return Err(Bad::Damaged);
    }
    let mut kept = Zeroizing::new(String::with_capacity(value.len()));
    kept.push_str(value);
    Ok(Field {
        label,
        value: Redacted::new(kept),
        hidden,
    })
}

/// A saved line: a text with no hidden mark, of at most `MAX_FIELD_CHARS` characters.
fn line<'a>(r: &mut Reader<'a>) -> Result<&'a str, Bad> {
    let text = r.text(MAX_TEXT_BYTES)?;
    held(text, MAX_FIELD_CHARS).map_err(|_| Bad::Damaged)?;
    Ok(text)
}
