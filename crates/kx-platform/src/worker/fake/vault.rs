//! The vault job as the script fakes it: any answer made for each ask's id, or a fixed fault.

use super::{Call, Faked};
use crate::worker::{Heard, VaultFrames};
use kx_ipc::{Fault, IpcError, VaultAsk, VaultReply};

/// What the vault worker does with the next ask; with none queued it fails inside.
#[derive(Clone, Debug)]
pub enum VaultAct {
    /// The reply this makes for the ask's id.
    Answer(fn(u32) -> VaultReply),
    /// A failure for the ask's id.
    Fail(Fault),
    /// This reply, whatever the ask's id.
    Raw(VaultReply),
    /// Nothing within the wait.
    Silent,
    /// A frame that could not be read.
    Broken(IpcError),
}

impl Faked for VaultFrames {
    type Act = VaultAct;

    fn usual() -> VaultAct {
        VaultAct::Fail(Fault::Internal)
    }

    fn note(ask: VaultAsk<'static>) -> Option<(u32, Call)> {
        match ask {
            VaultAsk::Bye => None,
            ask => Some((ask.id(), Call::Vault(ask))),
        }
    }

    fn answer(act: VaultAct, id: u32) -> Heard<VaultReply> {
        match act {
            VaultAct::Answer(make) => Heard::Frame(Ok(make(id))),
            VaultAct::Fail(fault) => Heard::Frame(Ok(VaultReply::Failed { id, fault })),
            VaultAct::Raw(reply) => Heard::Frame(Ok(reply)),
            VaultAct::Silent => Heard::Late,
            VaultAct::Broken(e) => Heard::Frame(Err(e)),
        }
    }
}
