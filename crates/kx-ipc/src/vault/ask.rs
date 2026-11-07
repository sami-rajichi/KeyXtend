//! What the host sends the vault job: make a padlock, seal parts, open a box, check a password, read a `KeePass` file,
//! or goodbye.

use super::body::{put_padlock, read_padlock};
use super::wire::{BYTES, KEEPASS, LOCKED, PASSWORD, Payload, Put, Sink, Size};
use super::{BoxPart, Wiped};
use crate::code::Kind;
use crate::error::IpcError;
use crate::frame::{Head, write_head};
use crate::limits::{MAX_BOX_WORK, MIN_BOX_WORK};
use std::borrow::Cow;
use std::fmt;
use std::io::{Read, Write};

/// A message from the host to the vault job; it borrows its bytes when written and owns them when read.
#[derive(Clone, PartialEq, Eq)]
pub enum VaultAsk<'a> {
    /// Make a padlock: a new identity locked by `password` at scrypt work factor `work` (log N).
    Padlock {
        /// The request, which the reply repeats.
        id: u32,
        /// From `MIN_BOX_WORK` to `MAX_BOX_WORK`.
        work: u8,
        /// The backup password, as UTF-8.
        password: Wiped<'a>,
    },
    /// Seal `parts` into a box under a padlock; it needs no password.
    Seal {
        /// The request, which the reply repeats.
        id: u32,
        /// The open padlock: an `age` X25519 recipient.
        recipient: Cow<'a, str>,
        /// The padlock's locked identity, which the box carries.
        locked: Cow<'a, [u8]>,
        /// Each known part once, at least one.
        parts: Vec<BoxPart<'a>>,
    },
    /// Open a box with `password`.
    Open {
        /// The request, which the reply repeats.
        id: u32,
        /// The backup password, as UTF-8.
        password: Wiped<'a>,
        /// The box, from 1 to `MAX_BOX_BYTES` bytes.
        bytes: Cow<'a, [u8]>,
    },
    /// Say whether `password` opens a padlock's locked identity.
    Check {
        /// The request, which the reply repeats.
        id: u32,
        /// The backup password, as UTF-8.
        password: Wiped<'a>,
        /// The locked identity.
        locked: Cow<'a, [u8]>,
    },
    /// Read a `KeePass` file's entries with `password`.
    Keepass {
        /// The request, which the reply repeats.
        id: u32,
        /// The file's password, as UTF-8.
        password: Wiped<'a>,
        /// The file, from 1 to `MAX_KEEPASS_BYTES` bytes.
        bytes: Cow<'a, [u8]>,
    },
    /// End: the vault job exits after this frame.
    Bye,
}

/// The kinds a vault ask may be.
pub(crate) const KINDS: &[Kind] = &[
    Kind::MakePadlock,
    Kind::Seal,
    Kind::OpenBox,
    Kind::CheckPassword,
    Kind::ReadKeepass,
    Kind::VaultBye,
];

impl VaultAsk<'_> {
    /// Reads one ask from the host.
    ///
    /// # Errors
    /// Any `IpcError`, `Closed` when the host ended between frames; the stream is out of step after any other.
    pub fn read<R: Read + ?Sized>(r: &mut R) -> Result<VaultAsk<'static>, IpcError> {
        let head = Head::read(r, KINDS)?;
        let (id, mut p) = (head.id, Payload::new(r, head.len));
        let ask = match head.kind {
            Kind::MakePadlock => {
                let work = check_work(p.byte()?)?;
                let password = read_password(&mut p)?;
                VaultAsk::Padlock { id, work, password }
            }
            Kind::Seal => {
                let (recipient, locked, parts) = read_padlock(&mut p)?;
                let (recipient, locked) = (Cow::Owned(recipient), Cow::Owned(locked));
                VaultAsk::Seal {
                    id,
                    recipient,
                    locked,
                    parts,
                }
            }
            Kind::OpenBox | Kind::ReadKeepass => read_file(&mut p, head)?,
            Kind::CheckPassword => {
                let password = read_password(&mut p)?;
                let locked = Cow::Owned(p.bytes(LOCKED)?);
                VaultAsk::Check {
                    id,
                    password,
                    locked,
                }
            }
            Kind::VaultBye if id == 0 => VaultAsk::Bye,
            Kind::VaultBye => return Err(IpcError::Inconsistent),
            _ => return Err(IpcError::BadKind),
        };
        p.end()?;
        Ok(ask)
    }

    /// Writes this ask as one frame, then flushes.
    ///
    /// # Errors
    /// `BadField`, `TooLong` or `Inconsistent`, before any byte is written, for an ask the job would refuse; `Io`
    /// when the stream fails.
    pub fn write<W: Write + ?Sized>(&self, w: &mut W) -> Result<(), IpcError> {
        let mut size = Size::default();
        self.put(&mut size)?;
        write_head(w, self.kind(), self.id(), size.0)?;
        self.put(&mut Put(w))?;
        w.flush()?;
        Ok(())
    }

    /// Runs this ask's payload through `s`.
    fn put(&self, s: &mut dyn Sink) -> Result<(), IpcError> {
        match self {
            Self::Padlock { work, password, .. } => {
                s.byte(check_work(*work)?)?;
                put_password(s, password)
            }
            Self::Seal {
                recipient,
                locked,
                parts,
                ..
            } => put_padlock(s, recipient, locked, parts),
            Self::Open {
                password, bytes, ..
            } => {
                put_password(s, password)?;
                s.field(BYTES, bytes)
            }
            Self::Keepass {
                password, bytes, ..
            } => {
                put_password(s, password)?;
                s.field(KEEPASS, bytes)
            }
            Self::Check {
                password, locked, ..
            } => {
                put_password(s, password)?;
                s.field(LOCKED, locked)
            }
            Self::Bye => Ok(()),
        }
    }

    fn kind(&self) -> Kind {
        match self {
            Self::Padlock { .. } => Kind::MakePadlock,
            Self::Seal { .. } => Kind::Seal,
            Self::Open { .. } => Kind::OpenBox,
            Self::Check { .. } => Kind::CheckPassword,
            Self::Keepass { .. } => Kind::ReadKeepass,
            Self::Bye => Kind::VaultBye,
        }
    }

    /// The request's id; goodbye's is 0.
    #[must_use]
    pub const fn id(&self) -> u32 {
        match self {
            Self::Padlock { id, .. }
            | Self::Seal { id, .. }
            | Self::Open { id, .. }
            | Self::Check { id, .. }
            | Self::Keepass { id, .. } => *id,
            Self::Bye => 0,
        }
    }
}

/// `work` when it is within the `box_work` setting's range.
fn check_work(work: u8) -> Result<u8, IpcError> {
    if (MIN_BOX_WORK..=MAX_BOX_WORK).contains(&work) {
        Ok(work)
    } else {
        Err(IpcError::BadField)
    }
}

/// Sends a password, refusing one that is not UTF-8.
fn put_password(s: &mut dyn Sink, password: &[u8]) -> Result<(), IpcError> {
    std::str::from_utf8(password).map_err(|_| IpcError::BadField)?;
    s.field(PASSWORD, password)
}

/// Reads a password into a wiped buffer, refusing one that is not UTF-8.
fn read_password<R: Read + ?Sized>(p: &mut Payload<'_, R>) -> Result<Wiped<'static>, IpcError> {
    let n = p.count(PASSWORD)?;
    let password = p.wiped(n)?;
    std::str::from_utf8(&password).map_err(|_| IpcError::BadField)?;
    Ok(Wiped::Kept(password))
}

/// Reads an open or a `KeePass` ask: a password, then the file.
fn read_file<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
    head: Head,
) -> Result<VaultAsk<'static>, IpcError> {
    let password = read_password(p)?;
    let box_kind = head.kind == Kind::OpenBox;
    let bytes = Cow::Owned(p.bytes(if box_kind { BYTES } else { KEEPASS })?);
    let id = head.id;
    Ok(if box_kind {
        VaultAsk::Open {
            id,
            password,
            bytes,
        }
    } else {
        VaultAsk::Keepass {
            id,
            password,
            bytes,
        }
    })
}

impl fmt::Debug for VaultAsk<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = self.id();
        match self {
            Self::Padlock { work, .. } => f
                .debug_struct("Padlock")
                .field("id", &id)
                .field("work", work)
                .finish_non_exhaustive(),
            Self::Seal { locked, parts, .. } => f
                .debug_struct("Seal")
                .field("id", &id)
                .field("locked", &locked.len())
                .field("parts", parts)
                .finish_non_exhaustive(),
            Self::Open { bytes, .. } | Self::Keepass { bytes, .. } => {
                let name = if matches!(self, Self::Open { .. }) {
                    "Open"
                } else {
                    "Keepass"
                };
                f.debug_struct(name)
                    .field("id", &id)
                    .field("len", &bytes.len())
                    .finish_non_exhaustive()
            }
            Self::Check { locked, .. } => f
                .debug_struct("Check")
                .field("id", &id)
                .field("locked", &locked.len())
                .finish_non_exhaustive(),
            Self::Bye => f.write_str("Bye"),
        }
    }
}
