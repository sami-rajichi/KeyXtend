//! The vault job's frames (ADR-0038 decision 6): padlocks, boxes, password checks and `KeePass` entries, on the media
//! frames' header with kinds of their own, so neither job reads the other's frames. Passwords and values travel in
//! buffers that are wiped when dropped, and `Debug` never shows them.

pub(crate) mod ask;
pub(crate) mod body;
pub(crate) mod reply;
pub(crate) mod wire;

use crate::error::IpcError;
use crate::wiped::Wiped;
use std::borrow::Cow;
use std::fmt;
use zeroize::Zeroizing;

/// One named part of a box.
#[derive(Clone, PartialEq, Eq)]
pub struct BoxPart<'a> {
    /// Its name, one of `BOX_PARTS`.
    pub name: Cow<'a, str>,
    /// Its plain bytes.
    pub bytes: Wiped<'a>,
}

impl fmt::Debug for BoxPart<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoxPart")
            .field("name", &self.name)
            .field("len", &self.bytes.len())
            .finish()
    }
}

/// One entry of a `KeePass` file; a field it lacks is empty.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct KeepassEntry {
    /// Its title.
    pub title: Zeroizing<String>,
    /// Its user name.
    pub user: Zeroizing<String>,
    /// Its password.
    pub password: Zeroizing<String>,
    /// Its address (URL).
    pub address: Zeroizing<String>,
    /// Its notes, line breaks and all.
    pub notes: Zeroizing<String>,
    /// Its custom strings.
    pub custom: Vec<KeepassField>,
}

impl KeepassEntry {
    /// The bytes this entry takes in a `KeePass` reply, which holds `MAX_ENTRIES_BYTES` of them at most.
    ///
    /// # Errors
    /// `TooLong` for a field past `MAX_FIELD_BYTES` or custom fields past `MAX_FIELDS`.
    pub fn wire_len(&self) -> Result<usize, IpcError> {
        let mut size = wire::Size::default();
        body::put_entry(&mut size, self)?;
        Ok(size.0)
    }
}

impl fmt::Debug for KeepassEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeepassEntry")
            .field("custom", &self.custom.len())
            .finish_non_exhaustive()
    }
}

/// One custom string of a `KeePass` entry.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct KeepassField {
    /// Its name.
    pub label: Zeroizing<String>,
    /// Its value.
    pub value: Zeroizing<String>,
}

impl fmt::Debug for KeepassField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("KeepassField")
    }
}

/// What the vault job left out of a `KeePass` import to keep within the caps.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Dropped {
    /// Entries past `MAX_ENTRIES` or the reply's size.
    pub entries: u32,
    /// Fields past `MAX_FIELD_BYTES`, and custom fields past `MAX_FIELDS`.
    pub fields: u32,
}
