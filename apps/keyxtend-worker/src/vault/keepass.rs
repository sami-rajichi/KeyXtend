//! `KeePass` import (ADR-0038 decision 6): a KDBX 3 or 4 file opened with its password alone, once its key derivation
//! has passed the vault job's caps. Entries come back group by group in the file's order, the recycle bin left out;
//! entries and fields past the caps are dropped and counted.

mod kdf;

use keepass::config::DatabaseVersion;
use keepass::db::{Entry, fields};
use keepass::error::{
    CryptographyError, DatabaseKeyError, DatabaseOpenError, DatabaseVersionParseError,
};
use keepass::{Database, DatabaseKey};
use kx_ipc::{
    Dropped, Fault, KeepassEntry, KeepassField, MAX_ENTRIES, MAX_ENTRIES_BYTES, MAX_FIELD_BYTES,
    MAX_FIELDS, MAX_KEEPASS_BYTES,
};
use zeroize::Zeroizing;

/// The caps an import keeps within.
#[derive(Clone, Copy)]
struct Caps {
    /// Entries, at most.
    entries: usize,
    /// Bytes in one field.
    field: usize,
    /// Custom fields in one entry.
    fields: usize,
    /// Bytes the entries take in the reply.
    bytes: usize,
}

/// The `KeePass` reply's caps.
const CAPS: Caps = Caps {
    entries: MAX_ENTRIES,
    field: MAX_FIELD_BYTES,
    fields: MAX_FIELDS,
    bytes: MAX_ENTRIES_BYTES,
};

/// The entries of `bytes`, a `KeePass` file opened with `password`, and what was dropped to keep within the caps.
///
/// # Errors
/// `TooLarge` past `MAX_KEEPASS_BYTES`; `Foreign` for a file that is not `KeePass`; `Unsupported` for `KeePass` 1, KDBX 2
/// or too heavy a key derivation, before any key is derived; `WrongPassword`, also for a file needing a key file.
pub fn read(bytes: &[u8], password: &str) -> Result<(Vec<KeepassEntry>, Dropped), Fault> {
    if bytes.len() > MAX_KEEPASS_BYTES {
        return Err(Fault::TooLarge);
    }
    let version = version(bytes)?;
    kdf::check(bytes)?;
    let key = DatabaseKey::new().with_password(password);
    let db = Database::parse(bytes, key).map_err(|e| open_fault(&e, &version))?;
    Ok(entries(&db, CAPS))
}

/// The file's version when it is KDBX 3 or 4.
fn version(bytes: &[u8]) -> Result<DatabaseVersion, Fault> {
    match DatabaseVersion::parse(bytes) {
        Ok(v @ (DatabaseVersion::KDB3(_) | DatabaseVersion::KDB4(_))) => Ok(v),
        Ok(_) | Err(DatabaseVersionParseError::InvalidKDBXVersion { .. }) => {
            Err(Fault::Unsupported)
        }
        Err(DatabaseVersionParseError::InvalidKDBXIdentifier) => Err(Fault::Foreign),
        Err(_) if kdf::cut_signature(bytes) => Err(Fault::Damaged),
        Err(_) => Err(Fault::Foreign),
    }
}

/// The reason an open failed; KDBX 3 checks no key before decrypting, so a wrong one shows as bad padding.
fn open_fault(e: &DatabaseOpenError, version: &DatabaseVersion) -> Fault {
    match e {
        DatabaseOpenError::Key(DatabaseKeyError::IncorrectKey) => Fault::WrongPassword,
        DatabaseOpenError::Cryptography(CryptographyError::InvalidPadding(_))
            if matches!(version, DatabaseVersion::KDB3(_)) =>
        {
            Fault::WrongPassword
        }
        DatabaseOpenError::UnsupportedVersion => Fault::Unsupported,
        DatabaseOpenError::VersionParse(_) => Fault::Foreign,
        _ => Fault::Damaged,
    }
}

/// Every entry within `caps`, a group's own before its subgroups', depth first, and what was dropped; the recycle bin
/// and all under it are left out.
fn entries(db: &Database, caps: Caps) -> (Vec<KeepassEntry>, Dropped) {
    let bin = db.recycle_bin().map(|g| g.id());
    let mut kept = Kept {
        caps,
        entries: Vec::new(),
        dropped: Dropped::default(),
        bytes: 0,
    };
    let mut ids = vec![db.root().id()];
    while let Some(id) = ids.pop() {
        let Some(group) = db.group(id).filter(|_| Some(id) != bin) else {
            continue;
        };
        group.entries().for_each(|e| kept.add(&e));
        let subgroups: Vec<_> = group.groups().map(|g| g.id()).collect();
        ids.extend(subgroups.into_iter().rev());
    }
    (kept.entries, kept.dropped)
}

/// The entries kept so far, what was dropped, and the bytes the kept ones take in the reply.
struct Kept {
    caps: Caps,
    entries: Vec<KeepassEntry>,
    dropped: Dropped,
    bytes: usize,
}

impl Kept {
    /// Keeps `e` with its fields capped, or drops it whole past the entry count or the reply's size.
    fn add(&mut self, e: &Entry) {
        if self.entries.len() >= self.caps.entries {
            return self.drop_one();
        }
        let (entry, lost) = entry(e, self.caps);
        let bytes = entry
            .wire_len()
            .map_or(usize::MAX, |n| self.bytes.saturating_add(n));
        if bytes > self.caps.bytes {
            return self.drop_one();
        }
        self.bytes = bytes;
        self.entries.push(entry);
        self.dropped.fields = self.dropped.fields.saturating_add(lost);
    }

    /// Counts one entry dropped.
    fn drop_one(&mut self) {
        self.dropped.entries = self.dropped.entries.saturating_add(1);
    }
}

/// One entry within `caps`: its five standard fields, one past the cap left empty, then its custom strings; and how
/// many fields it lost.
fn entry(e: &Entry, caps: Caps) -> (KeepassEntry, u32) {
    let mut lost = 0_u32;
    let mut field = |key: &str| {
        let value = e.get(key).unwrap_or_default();
        if value.len() > caps.field {
            lost = lost.saturating_add(1);
            Zeroizing::default()
        } else {
            Zeroizing::new(value.to_owned())
        }
    };
    let (title, user) = (field(fields::TITLE), field(fields::USERNAME));
    let (password, address) = (field(fields::PASSWORD), field(fields::URL));
    let notes = field(fields::NOTES);
    let (custom, more) = custom(e, caps);
    let entry = KeepassEntry {
        title,
        user,
        password,
        address,
        notes,
        custom,
    };
    (entry, lost.saturating_add(more))
}

/// An entry's custom strings A to Z by name, as the file keeps no order, leaving out those past `caps`; and how many.
fn custom(e: &Entry, caps: Caps) -> (Vec<KeepassField>, u32) {
    let mut lost = 0_usize;
    let mut kept = Vec::new();
    for (name, value) in &e.fields {
        let value = value.get();
        if fields::KNOWN_FIELDS.contains(&name.as_str()) {
            continue;
        }
        if name.len() > caps.field || value.len() > caps.field {
            lost += 1;
        } else {
            kept.push(KeepassField {
                label: Zeroizing::new(name.clone()),
                value: Zeroizing::new(value.clone()),
            });
        }
    }
    kept.sort_by(|a, b| a.label.cmp(&b.label));
    lost += kept.len().saturating_sub(caps.fields);
    kept.truncate(caps.fields);
    (kept, u32::try_from(lost).unwrap_or(u32::MAX))
}

#[cfg(test)]
mod tests;
