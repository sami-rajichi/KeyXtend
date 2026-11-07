//! The two jobs' frames from `kx-ipc`, as the host writes asks and reads replies.

use super::Frames;
use kx_ipc::{Ask, IpcError, MEDIA_JOB, Reply, VAULT_JOB, VaultAsk, VaultReply};
use std::io::{Read, Write};

/// The media job's frames: draws and DIBs.
#[derive(Debug)]
pub struct MediaFrames;

impl Frames for MediaFrames {
    const JOB: &'static str = MEDIA_JOB;
    type Ask = Ask<'static>;
    type Reply = Reply;

    fn bye() -> Ask<'static> {
        Ask::Bye
    }

    fn id(reply: &Reply) -> u32 {
        reply.id()
    }

    fn write<W: Write + ?Sized>(ask: &Ask<'static>, w: &mut W) -> Result<(), IpcError> {
        ask.write(w)
    }

    fn read<R: Read + ?Sized>(r: &mut R) -> Result<Reply, IpcError> {
        Reply::read(r)
    }
}

/// The vault job's frames: padlocks, boxes, password checks and `KeePass` files.
#[derive(Debug)]
pub struct VaultFrames;

impl Frames for VaultFrames {
    const JOB: &'static str = VAULT_JOB;
    type Ask = VaultAsk<'static>;
    type Reply = VaultReply;

    fn bye() -> VaultAsk<'static> {
        VaultAsk::Bye
    }

    fn id(reply: &VaultReply) -> u32 {
        reply.id()
    }

    fn write<W: Write + ?Sized>(ask: &VaultAsk<'static>, w: &mut W) -> Result<(), IpcError> {
        ask.write(w)
    }

    fn read<R: Read + ?Sized>(r: &mut R) -> Result<VaultReply, IpcError> {
        VaultReply::read(r)
    }
}
