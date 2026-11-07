//! The bodies both vault sides send: a padlock with its part list, and `KeePass` entries.

use super::wire::{
    CUSTOM, DROPPED, ENTRIES, FIELD, LOCKED, NAME, PART, PARTS, Payload, RECIPIENT, Sink,
};
use super::{BoxPart, Dropped, KeepassEntry, KeepassField, Wiped};
use crate::error::IpcError;
use crate::limits::{BOX_PARTS, MAX_BOX_BYTES};
use std::borrow::Cow;
use std::io::Read;
use zeroize::Zeroizing;

/// Sends a padlock, then its part list.
pub(crate) fn put_padlock(
    s: &mut dyn Sink,
    recipient: &str,
    locked: &[u8],
    parts: &[BoxPart<'_>],
) -> Result<(), IpcError> {
    s.field(RECIPIENT, recipient.as_bytes())?;
    s.field(LOCKED, locked)?;
    put_parts(s, parts)
}

/// Sends a part list: each part by its known name, once, with all their bytes within `MAX_BOX_BYTES`.
fn put_parts(s: &mut dyn Sink, parts: &[BoxPart<'_>]) -> Result<(), IpcError> {
    s.count(PARTS, parts.len())?;
    let mut seen = Vec::new();
    let mut total: usize = 0;
    for part in parts {
        seen.push(part_name(part.name.as_bytes(), &seen)?);
        total = total
            .checked_add(part.bytes.len())
            .ok_or(IpcError::TooLong)?;
        if total > MAX_BOX_BYTES {
            return Err(IpcError::TooLong);
        }
        s.field(NAME, part.name.as_bytes())?;
        s.field(PART, &part.bytes)?;
    }
    Ok(())
}

/// The known part `raw` names, `BadField` when it is unknown or already in `seen`.
pub(crate) fn part_name(raw: &[u8], seen: &[&str]) -> Result<&'static str, IpcError> {
    let name = BOX_PARTS.iter().find(|n| n.as_bytes() == raw);
    match name {
        Some(name) if !seen.contains(name) => Ok(name),
        _ => Err(IpcError::BadField),
    }
}

/// Reads a padlock and its part list.
pub(crate) fn read_padlock<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
) -> Result<(String, Vec<u8>, Vec<BoxPart<'static>>), IpcError> {
    let recipient = p.text(RECIPIENT)?;
    let locked = p.bytes(LOCKED)?;
    let count = p.count(PARTS)?;
    let (mut parts, mut seen) = (Vec::new(), Vec::new());
    let mut total: usize = 0;
    for _ in 0..count {
        let name = part_name(&p.bytes(NAME)?, &seen)?;
        seen.push(name);
        let len = p.count(PART)?;
        total = total.saturating_add(len);
        if total > MAX_BOX_BYTES {
            return Err(IpcError::TooLong);
        }
        let bytes = Wiped::Kept(p.wiped(len)?);
        parts.push(BoxPart {
            name: Cow::Borrowed(name),
            bytes,
        });
    }
    Ok((recipient, locked, parts))
}

/// Sends a `KeePass` import: what it dropped, the entries' count, then each entry's five fields and custom fields.
pub(crate) fn put_entries(
    s: &mut dyn Sink,
    entries: &[KeepassEntry],
    dropped: Dropped,
) -> Result<(), IpcError> {
    s.count(DROPPED, widen(dropped.entries))?;
    s.count(DROPPED, widen(dropped.fields))?;
    s.count(ENTRIES, entries.len())?;
    entries.iter().try_for_each(|e| put_entry(s, e))
}

/// Sends one entry: its five fields, then its custom fields.
pub(crate) fn put_entry(s: &mut dyn Sink, e: &KeepassEntry) -> Result<(), IpcError> {
    for field in [&e.title, &e.user, &e.password, &e.address, &e.notes] {
        s.field(FIELD, field.as_bytes())?;
    }
    s.count(CUSTOM, e.custom.len())?;
    for f in &e.custom {
        s.field(FIELD, f.label.as_bytes())?;
        s.field(FIELD, f.value.as_bytes())?;
    }
    Ok(())
}

/// A count as `usize`, which holds any `u32` on the targets we build.
fn widen(n: u32) -> usize {
    usize::try_from(n).unwrap_or(usize::MAX)
}

/// Reads a `KeePass` import's dropped counts.
pub(crate) fn read_dropped<R: Read + ?Sized>(p: &mut Payload<'_, R>) -> Result<Dropped, IpcError> {
    let mut count = || u32::try_from(p.count(DROPPED)?).map_err(|_| IpcError::TooLong);
    let entries = count()?;
    Ok(Dropped {
        entries,
        fields: count()?,
    })
}

/// Reads `KeePass` entries.
pub(crate) fn read_entries<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
) -> Result<Vec<KeepassEntry>, IpcError> {
    let count = p.count(ENTRIES)?;
    let mut entries = Vec::new();
    for _ in 0..count {
        let mut field = || p.secret(FIELD);
        let (title, user, password) = (field()?, field()?, field()?);
        let (address, notes) = (field()?, field()?);
        let custom = (0..p.count(CUSTOM)?)
            .map(|_| read_field(p))
            .collect::<Result<_, _>>()?;
        entries.push(KeepassEntry {
            title,
            user,
            password,
            address,
            notes,
            custom,
        });
    }
    Ok(entries)
}

/// Reads one custom field: its label, then its value.
fn read_field<R: Read + ?Sized>(p: &mut Payload<'_, R>) -> Result<KeepassField, IpcError> {
    let label: Zeroizing<String> = p.secret(FIELD)?;
    let value = p.secret(FIELD)?;
    Ok(KeepassField { label, value })
}
