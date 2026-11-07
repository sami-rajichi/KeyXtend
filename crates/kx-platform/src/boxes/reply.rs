//! The vault job's replies as the port's results: values move into wiped buffers without a copy, a fault becomes the
//! port's error where that call has it and `Failed` elsewhere, and a reply of another kind is `None`, which ends the
//! worker.

use crate::worker::Lost;
use kx_ipc::{BoxPart as Part, Fault, KeepassEntry as Entry, KeepassField as Field};
use kx_ipc::{VaultReply, Wiped};
use kx_module_api::ports::{
    BoxError, BoxPart, KeepassEntry, KeepassField, KeepassImport, Opened, Padlock,
};
use kx_module_api::{Redacted, Zeroizing};

/// What a reply gives: its answer, the worker's refusal, or `None` when it does not fit the ask.
type Taken<T> = Option<Result<T, BoxError>>;

/// Every error a call that reads a box or a file has.
const READS: &[BoxError] = &[
    BoxError::WrongPassword,
    BoxError::Damaged,
    BoxError::Unsupported,
    BoxError::TooLarge,
];

/// A new padlock.
pub(super) fn padlock(reply: VaultReply) -> Taken<Padlock> {
    match reply {
        VaultReply::Padlock {
            recipient, locked, ..
        } => Some(Ok(lock(recipient, locked))),
        reply => refused(&reply, &[BoxError::TooLarge]),
    }
}

/// A sealed box's bytes.
pub(super) fn sealed(reply: VaultReply) -> Taken<Redacted<Vec<u8>>> {
    match reply {
        VaultReply::Sealed { bytes, .. } => Some(Ok(Redacted::new(bytes))),
        reply => refused(&reply, &[BoxError::Damaged, BoxError::TooLarge]),
    }
}

/// An opened box: its own padlock and its parts.
pub(super) fn opened(reply: VaultReply) -> Taken<Opened> {
    match reply {
        VaultReply::Opened {
            recipient,
            locked,
            parts,
            ..
        } => {
            let parts = parts.into_iter().map(part).collect();
            let padlock = lock(recipient, locked);
            Some(Ok(Opened { padlock, parts }))
        }
        reply => refused(&reply, READS),
    }
}

/// Whether the password opens the padlock.
pub(super) fn checked(reply: VaultReply) -> Taken<bool> {
    match reply {
        VaultReply::Checked { yes, .. } => Some(Ok(yes)),
        reply => refused(&reply, &[BoxError::Damaged]),
    }
}

/// A `KeePass` file's entries and what was dropped.
pub(super) fn entries(reply: VaultReply) -> Taken<KeepassImport> {
    match reply {
        VaultReply::Entries {
            entries, dropped, ..
        } => {
            let entries = entries.into_iter().map(entry).collect();
            Some(Ok(KeepassImport { entries, dropped }))
        }
        reply => refused(&reply, READS),
    }
}

/// The worker's refusal in `reply` as the port's error, `Failed` when the call `has` no such error; `None` for any
/// other kind of reply.
fn refused<T>(reply: &VaultReply, has: &[BoxError]) -> Taken<T> {
    let &VaultReply::Failed { fault, .. } = reply else {
        return None;
    };
    let error = error(fault);
    Some(Err(if has.contains(&error) {
        error
    } else {
        BoxError::Failed
    }))
}

/// The port's error for the worker's fault.
const fn error(fault: Fault) -> BoxError {
    match fault {
        Fault::WrongPassword => BoxError::WrongPassword,
        Fault::Damaged | Fault::Foreign => BoxError::Damaged,
        Fault::TooLarge => BoxError::TooLarge,
        Fault::Unsupported => BoxError::Unsupported,
        Fault::Internal => BoxError::Failed,
    }
}

impl From<Lost> for BoxError {
    /// A late worker is `TimedOut`; one that ended, lied or does not run is `Failed`.
    fn from(lost: Lost) -> Self {
        match lost {
            Lost::Late => Self::TimedOut,
            Lost::Gone | Lost::Down { .. } => Self::Failed,
        }
    }
}

/// The padlock of `recipient` and its `locked` identity.
fn lock(recipient: String, locked: Vec<u8>) -> Padlock {
    let (recipient, locked) = (Redacted::new(recipient), Redacted::new(locked));
    Padlock { recipient, locked }
}

/// An opened part, its bytes kept in their wiped buffer.
fn part(p: Part<'static>) -> BoxPart {
    let bytes = match p.bytes {
        Wiped::Kept(bytes) => bytes,
        Wiped::Lent(bytes) => Zeroizing::new(bytes.to_vec()),
    };
    let name = p.name.into_owned();
    BoxPart {
        name,
        bytes: Redacted::new(bytes),
    }
}

/// A `KeePass` entry, each value kept in its wiped buffer.
fn entry(e: Entry) -> KeepassEntry {
    KeepassEntry {
        title: Redacted::new(e.title),
        user: Redacted::new(e.user),
        password: Redacted::new(e.password),
        address: Redacted::new(e.address),
        notes: Redacted::new(e.notes),
        custom: e.custom.into_iter().map(field).collect(),
    }
}

/// A custom field: its name, which the port keeps unwiped, and its wiped value.
fn field(mut f: Field) -> KeepassField {
    let label = Redacted::new(std::mem::take(&mut *f.label));
    KeepassField {
        label,
        value: Redacted::new(f.value),
    }
}
