//! What the vault job sends back: a padlock, a sealed box, an opened box, a yes or no, `KeePass` entries, or a fixed
//! reason it made none.

use super::body::{put_entries, put_padlock, read_dropped, read_entries, read_padlock};
use super::wire::{BYTES, LOCKED, Payload, Put, RECIPIENT, Sink, Size};
use super::{BoxPart, Dropped, KeepassEntry};
use crate::code::{Code, Fault, Kind};
use crate::error::IpcError;
use crate::frame::{Head, check_len, write_head};
use std::fmt;
use std::io::{Read, Write};

/// A message from the vault job to the host, answering one ask.
#[derive(Clone, PartialEq, Eq)]
pub enum VaultReply {
    /// A new padlock.
    Padlock {
        /// The ask this answers.
        id: u32,
        /// The open padlock: an `age` X25519 recipient.
        recipient: String,
        /// The identity, locked by scrypt under the password.
        locked: Vec<u8>,
    },
    /// A sealed box.
    Sealed {
        /// The ask this answers.
        id: u32,
        /// The box, from 1 to `MAX_BOX_BYTES` bytes.
        bytes: Vec<u8>,
    },
    /// An opened box: its own padlock, then its parts.
    Opened {
        /// The ask this answers.
        id: u32,
        /// The box's open padlock.
        recipient: String,
        /// The box's locked identity.
        locked: Vec<u8>,
        /// Its parts, in the box's order.
        parts: Vec<BoxPart<'static>>,
    },
    /// Whether the password opens the locked identity.
    Checked {
        /// The ask this answers.
        id: u32,
        /// True when it does.
        yes: bool,
    },
    /// A `KeePass` file's entries, in the file's order, and what was dropped to keep within the caps.
    Entries {
        /// The ask this answers.
        id: u32,
        /// At most `MAX_ENTRIES`.
        entries: Vec<KeepassEntry>,
        /// The entries and fields left out.
        dropped: Dropped,
    },
    /// Nothing made, for a fixed reason.
    Failed {
        /// The ask this answers.
        id: u32,
        /// Why.
        fault: Fault,
    },
}

/// The kinds a vault reply may be.
pub(crate) const KINDS: &[Kind] = &[
    Kind::Padlock,
    Kind::Sealed,
    Kind::Opened,
    Kind::Checked,
    Kind::Entries,
    Kind::VaultFailed,
];

impl VaultReply {
    /// Reads one reply from the vault job.
    ///
    /// # Errors
    /// Any `IpcError`, `Closed` when the job ended between frames; the stream is out of step after any other.
    pub fn read<R: Read + ?Sized>(r: &mut R) -> Result<Self, IpcError> {
        let head = Head::read(r, KINDS)?;
        let mut p = Payload::new(r, head.len);
        let reply = read_body(&mut p, head)?;
        p.end()?;
        Ok(reply)
    }

    /// Writes this reply as one frame, then flushes.
    ///
    /// # Errors
    /// As `check`, before any byte is written; `Io` when the stream fails.
    pub fn write<W: Write + ?Sized>(&self, w: &mut W) -> Result<(), IpcError> {
        let len = self.size()?;
        write_head(w, self.kind(), self.id(), len)?;
        self.put(&mut Put(w))?;
        w.flush()?;
        Ok(())
    }

    /// Checks this reply as `write` does, so the job can answer `TooLarge` for one past the caps.
    ///
    /// # Errors
    /// `TooLong` past a cap, `BadField` for an unknown or repeated part, and `Inconsistent` for an empty field.
    pub fn check(&self) -> Result<(), IpcError> {
        check_len(self.kind(), self.size()?)
    }

    /// The ask this reply answers.
    #[must_use]
    pub const fn id(&self) -> u32 {
        match self {
            Self::Padlock { id, .. }
            | Self::Sealed { id, .. }
            | Self::Opened { id, .. }
            | Self::Checked { id, .. }
            | Self::Entries { id, .. }
            | Self::Failed { id, .. } => *id,
        }
    }

    /// The payload's length, once every field is checked.
    fn size(&self) -> Result<usize, IpcError> {
        let mut size = Size::default();
        self.put(&mut size)?;
        Ok(size.0)
    }

    /// Runs this reply's payload through `s`.
    fn put(&self, s: &mut dyn Sink) -> Result<(), IpcError> {
        match self {
            Self::Padlock {
                recipient, locked, ..
            } => {
                s.field(RECIPIENT, recipient.as_bytes())?;
                s.field(LOCKED, locked)
            }
            Self::Sealed { bytes, .. } => s.field(BYTES, bytes),
            Self::Opened {
                recipient,
                locked,
                parts,
                ..
            } => put_padlock(s, recipient, locked, parts),
            Self::Checked { yes, .. } => s.byte(u8::from(*yes)),
            Self::Entries {
                entries, dropped, ..
            } => put_entries(s, entries, *dropped),
            Self::Failed { fault, .. } => s.byte(fault.code()),
        }
    }

    const fn kind(&self) -> Kind {
        match self {
            Self::Padlock { .. } => Kind::Padlock,
            Self::Sealed { .. } => Kind::Sealed,
            Self::Opened { .. } => Kind::Opened,
            Self::Checked { .. } => Kind::Checked,
            Self::Entries { .. } => Kind::Entries,
            Self::Failed { .. } => Kind::VaultFailed,
        }
    }
}

/// Reads the payload of a reply of `head`'s kind.
fn read_body<R: Read + ?Sized>(p: &mut Payload<'_, R>, head: Head) -> Result<VaultReply, IpcError> {
    let id = head.id;
    Ok(match head.kind {
        Kind::Padlock => {
            let recipient = p.text(RECIPIENT)?;
            let locked = p.bytes(LOCKED)?;
            VaultReply::Padlock {
                id,
                recipient,
                locked,
            }
        }
        Kind::Sealed => VaultReply::Sealed {
            id,
            bytes: p.bytes(BYTES)?,
        },
        Kind::Opened => {
            let (recipient, locked, parts) = read_padlock(p)?;
            VaultReply::Opened {
                id,
                recipient,
                locked,
                parts,
            }
        }
        Kind::Checked => {
            let yes = yes(p.byte()?)?;
            VaultReply::Checked { id, yes }
        }
        Kind::Entries => read_import(p, id)?,
        Kind::VaultFailed => {
            let fault = Fault::from_code(p.byte()?).ok_or(IpcError::BadField)?;
            VaultReply::Failed { id, fault }
        }
        _ => return Err(IpcError::BadKind),
    })
}

/// Reads a `KeePass` import: what it dropped, then its entries.
fn read_import<R: Read + ?Sized>(p: &mut Payload<'_, R>, id: u32) -> Result<VaultReply, IpcError> {
    let dropped = read_dropped(p)?;
    let entries = read_entries(p)?;
    Ok(VaultReply::Entries {
        id,
        entries,
        dropped,
    })
}

/// A checked byte as yes or no, refusing any other.
fn yes(byte: u8) -> Result<bool, IpcError> {
    match byte {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(IpcError::BadField),
    }
}

impl fmt::Debug for VaultReply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = self.id();
        match self {
            Self::Padlock { locked, .. } => f
                .debug_struct("Padlock")
                .field("id", &id)
                .field("locked", &locked.len())
                .finish_non_exhaustive(),
            Self::Sealed { bytes, .. } => f
                .debug_struct("Sealed")
                .field("id", &id)
                .field("len", &bytes.len())
                .finish(),
            Self::Opened { parts, .. } => f
                .debug_struct("Opened")
                .field("id", &id)
                .field("parts", parts)
                .finish_non_exhaustive(),
            Self::Checked { yes, .. } => f
                .debug_struct("Checked")
                .field("id", &id)
                .field("yes", yes)
                .finish(),
            Self::Entries {
                entries, dropped, ..
            } => f
                .debug_struct("Entries")
                .field("id", &id)
                .field("count", &entries.len())
                .field("dropped", dropped)
                .finish(),
            Self::Failed { fault, .. } => f
                .debug_struct("Failed")
                .field("id", &id)
                .field("fault", fault)
                .finish(),
        }
    }
}
