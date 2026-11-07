//! The vault job (ADR-0038 decision 6): it makes padlocks, seals and opens backup boxes, checks passwords and reads
//! `KeePass` files, so neither format is parsed in the uiAccess keyboard. Every reply is checked against the frame's
//! caps before it is sent, and one past them is `TooLarge`.

mod boxes;
mod keepass;

use crate::serve::{Answer, Asked};
use kx_ipc::{Fault, IpcError, VaultAsk, VaultReply, Wiped};
use std::io::{Read, Write};

impl Asked for VaultAsk<'static> {
    type Reply = VaultReply;

    fn next<R: Read + ?Sized>(r: &mut R) -> Result<Self, IpcError> {
        VaultAsk::read(r)
    }

    fn id(&self) -> Option<u32> {
        (!matches!(self, VaultAsk::Bye)).then(|| VaultAsk::id(self))
    }
}

impl Answer for VaultReply {
    fn internal(id: u32) -> Self {
        VaultReply::Failed {
            id,
            fault: Fault::Internal,
        }
    }

    fn send<W: Write + ?Sized>(&self, w: &mut W) -> Result<(), IpcError> {
        self.write(w)
    }
}

/// Answers one vault ask; goodbye is never answered, as the loop ends first.
pub fn reply(ask: VaultAsk<'_>) -> VaultReply {
    let id = ask.id();
    make(ask)
        .and_then(fit)
        .unwrap_or_else(|fault| VaultReply::Failed { id, fault })
}

/// What `ask` makes, or why it makes nothing.
fn make(ask: VaultAsk<'_>) -> Result<VaultReply, Fault> {
    let id = ask.id();
    Ok(match ask {
        VaultAsk::Padlock { work, password, .. } => {
            padlocked(id, boxes::padlock(text(&password)?, work)?)
        }
        VaultAsk::Seal {
            recipient,
            locked,
            parts,
            ..
        } => {
            let bytes = boxes::seal(&recipient, &locked, &parts)?;
            VaultReply::Sealed { id, bytes }
        }
        VaultAsk::Open {
            password, bytes, ..
        } => opened(id, boxes::open(&bytes, text(&password)?)?),
        VaultAsk::Check {
            password, locked, ..
        } => {
            let yes = boxes::check(&locked, text(&password)?)?;
            VaultReply::Checked { id, yes }
        }
        VaultAsk::Keepass {
            password, bytes, ..
        } => {
            let (entries, dropped) = keepass::read(&bytes, text(&password)?)?;
            VaultReply::Entries {
                id,
                entries,
                dropped,
            }
        }
        VaultAsk::Bye => return Err(Fault::Internal),
    })
}

/// The reply carrying padlock `p`.
fn padlocked(id: u32, p: boxes::Padlock) -> VaultReply {
    VaultReply::Padlock {
        id,
        recipient: p.recipient,
        locked: p.locked,
    }
}

/// The reply carrying opened box `o`.
fn opened(id: u32, o: boxes::Opened) -> VaultReply {
    VaultReply::Opened {
        id,
        recipient: o.padlock.recipient,
        locked: o.padlock.locked,
        parts: o.parts,
    }
}

/// A password as text; the frame has already refused any that is not UTF-8.
fn text<'a>(password: &'a Wiped<'_>) -> Result<&'a str, Fault> {
    std::str::from_utf8(password).map_err(|_| Fault::Internal)
}

/// `reply` when it is within the frame's caps: `TooLarge` past them, and `Internal` for any other fault in it.
fn fit(reply: VaultReply) -> Result<VaultReply, Fault> {
    match reply.check() {
        Ok(()) => Ok(reply),
        Err(IpcError::TooLong) => Err(Fault::TooLarge),
        Err(_) => Err(Fault::Internal),
    }
}

#[cfg(test)]
mod tests;
