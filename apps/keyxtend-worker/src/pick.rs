//! The pick job (ADR-0056 decision 6): `keyxtend-worker pick` shows the system's Save and folder dialogs, one at a
//! time, so no shell extension loads into the app. It tells the path chosen only once it passes the ask's own check,
//! and a dialog that failed with the system's code.

use crate::serve::{self, Answer, Asked};
use kx_ipc::{IpcError, PickAsk, PickReply};
use kx_platform_record::pick::{Dialogs, Picked, Unshown};
use std::borrow::Cow;
use std::io::{Read, Write};

impl Asked for PickAsk<'static> {
    type Reply = PickReply<'static>;

    fn next<R: Read + ?Sized>(r: &mut R) -> Result<Self, IpcError> {
        Self::read(r)
    }

    fn id(&self) -> Option<u32> {
        match self {
            Self::Bye => None,
            ask => Some(ask.id()),
        }
    }
}

impl Answer for PickReply<'static> {
    fn internal(id: u32) -> Self {
        Self::Failed {
            id,
            code: Self::NO_CODE,
        }
    }

    fn send<W: Write + ?Sized>(&self, w: &mut W) -> Result<(), IpcError> {
        self.write(w)
    }
}

/// The pick job: `run` with every panic kept off stderr.
///
/// # Errors
/// As `run`.
pub fn serve<R, W>(r: &mut R, w: &mut W, dialogs: &impl Dialogs) -> Result<(), IpcError>
where
    R: Read + ?Sized,
    W: Write + ?Sized,
{
    serve::hush();
    run(r, w, dialogs)
}

/// Answers each ask read from `r` on `w` with one of `dialogs`, until goodbye or the end of input.
///
/// # Errors
/// The first frame that cannot be read or written.
pub fn run<R, W>(r: &mut R, w: &mut W, dialogs: &impl Dialogs) -> Result<(), IpcError>
where
    R: Read + ?Sized,
    W: Write + ?Sized,
{
    serve::serve(r, w, |ask: PickAsk<'static>| reply(&ask, dialogs))
}

/// Shows the dialog `ask` asks for and answers it. A path that fails `ask`'s check is refused and never told, so
/// nothing is ever written there.
fn reply(ask: &PickAsk<'_>, dialogs: &impl Dialogs) -> PickReply<'static> {
    let id = ask.id();
    let shown = match ask {
        PickAsk::Save {
            folder, name, kind, ..
        } => dialogs.save(folder, name, *kind),
        PickAsk::Folder { at, .. } => dialogs.folder(at),
        PickAsk::Bye => return PickReply::internal(id),
    };
    match shown {
        Ok(Picked::Path(path)) if ask.check(&path).is_ok() => PickReply::Picked {
            id,
            path: Cow::Owned(path),
        },
        Ok(Picked::Path(_)) => PickReply::Refused { id },
        Ok(Picked::Cancelled) => PickReply::Cancelled { id },
        Err(Unshown { code }) => PickReply::Failed { id, code },
    }
}

#[cfg(test)]
mod tests;
