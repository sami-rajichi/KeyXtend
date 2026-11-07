//! The vault asks, built from wiped copies, and refused as the port says before any worker starts when the frame would
//! refuse them.

use kx_ipc::{
    BOX_PARTS, BoxPart as Part, IpcError, MAX_BOX_BYTES, MAX_PASSWORD_BYTES, VaultAsk, Wiped,
};
use kx_module_api::ports::{BoxError, BoxPart, Padlock};
use kx_module_api::{Secret, Zeroizing};
use std::borrow::Cow;
use std::io;

/// A padlock ask; a work factor outside the frame's range is `TooLarge`.
pub(super) fn padlock(id: u32, password: Secret, work: u8) -> Result<VaultAsk<'static>, BoxError> {
    let password = wiped(password)?;
    fits(VaultAsk::Padlock { id, work, password }, |_| {
        BoxError::TooLarge
    })
}

/// A seal ask: no part or an unknown or repeated one is `Failed`, parts past `MAX_BOX_BYTES` are `TooLarge`, and a
/// padlock the frame will not carry is `Damaged`, as one that will not read.
pub(super) fn seal(
    id: u32,
    padlock: Padlock,
    parts: Vec<BoxPart>,
) -> Result<VaultAsk<'static>, BoxError> {
    named(&parts)?;
    let size = |p: &BoxPart| p.bytes.expose().len();
    let total = parts.iter().map(size).fold(0, usize::saturating_add);
    if total > MAX_BOX_BYTES {
        return Err(BoxError::TooLarge);
    }
    let parts = parts.into_iter().map(part).collect();
    let recipient = Cow::Owned(padlock.recipient.into_inner());
    let locked = Cow::Owned(padlock.locked.into_inner());
    let ask = VaultAsk::Seal {
        id,
        recipient,
        locked,
        parts,
    };
    fits(ask, |_| BoxError::Damaged)
}

/// An open ask: a box the frame will not carry is `TooLarge` or, empty, `Damaged`.
pub(super) fn open(
    id: u32,
    bytes: Vec<u8>,
    password: Secret,
) -> Result<VaultAsk<'static>, BoxError> {
    let password = wiped(password)?;
    let bytes = Cow::Owned(bytes);
    fits(
        VaultAsk::Open {
            id,
            password,
            bytes,
        },
        file,
    )
}

/// A check ask; a locked identity the frame will not carry is `Damaged`.
pub(super) fn check(
    id: u32,
    padlock: Padlock,
    password: Secret,
) -> Result<VaultAsk<'static>, BoxError> {
    let password = wiped(password)?;
    let locked = Cow::Owned(padlock.locked.into_inner());
    fits(
        VaultAsk::Check {
            id,
            password,
            locked,
        },
        |_| BoxError::Damaged,
    )
}

/// A `KeePass` ask: a file the frame will not carry is `TooLarge` or, empty, `Damaged`.
pub(super) fn keepass(
    id: u32,
    bytes: Vec<u8>,
    password: Secret,
) -> Result<VaultAsk<'static>, BoxError> {
    let password = wiped(password)?;
    let bytes = Cow::Owned(bytes);
    fits(
        VaultAsk::Keepass {
            id,
            password,
            bytes,
        },
        file,
    )
}

/// `ask` when the job would take it, by `kx-ipc`'s own writer checks; `refused` names the port's error otherwise.
fn fits(
    ask: VaultAsk<'static>,
    refused: impl FnOnce(IpcError) -> BoxError,
) -> Result<VaultAsk<'static>, BoxError> {
    ask.write(&mut io::sink()).map_err(refused)?;
    Ok(ask)
}

/// The port's error for a file the frame will not carry: too long is `TooLarge`, and an empty one is cut short.
fn file(e: IpcError) -> BoxError {
    if e == IpcError::TooLong {
        BoxError::TooLarge
    } else {
        BoxError::Damaged
    }
}

/// `password` as the frame's wiped bytes, moved without a copy; one past `MAX_PASSWORD_BYTES`, which no field
/// holds, is `Failed`.
fn wiped(password: Secret) -> Result<Wiped<'static>, BoxError> {
    let mut text = password.into_inner();
    if text.len() > MAX_PASSWORD_BYTES {
        return Err(BoxError::Failed);
    }
    let bytes = std::mem::take(&mut *text).into_bytes();
    Ok(Wiped::Kept(Zeroizing::new(bytes)))
}

/// `Failed` for no part or for one whose name is unknown or repeated: a module's slip, which the frame refuses.
fn named(parts: &[BoxPart]) -> Result<(), BoxError> {
    let mut seen: Vec<&str> = Vec::new();
    for name in parts.iter().map(|p| p.name.as_str()) {
        if !BOX_PARTS.contains(&name) || seen.contains(&name) {
            return Err(BoxError::Failed);
        }
        seen.push(name);
    }
    if seen.is_empty() {
        Err(BoxError::Failed)
    } else {
        Ok(())
    }
}

/// `p` as a frame carries it, its bytes still in their wiped buffer.
fn part(p: BoxPart) -> Part<'static> {
    let bytes = Wiped::Kept(p.bytes.into_inner());
    let name = Cow::Owned(p.name);
    Part { name, bytes }
}
