//! Each job's frames from `kx-ipc`: the media and vault jobs' asks and replies, and the voice and fetch jobs' asks and
//! the notes they send on their own.

use super::{Frames, Streams};
use kx_ipc::{
    Ask, FETCH_JOB, FetchAsk, FetchNote, IpcError, MEDIA_JOB, Reply, VAULT_JOB, VOICE_JOB,
    VaultAsk, VaultReply, VoiceAsk, VoiceNote,
};
use kx_module_api::ports::SpeechError;
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

/// The voice job's frames (ADR-0042): dictations, devices, facts and checks, and the notes the job sends on its own.
#[derive(Debug)]
pub struct VoiceFrames;

impl Frames for VoiceFrames {
    const JOB: &'static str = VOICE_JOB;
    type Ask = VoiceAsk<'static>;
    type Reply = VoiceNote;

    fn bye() -> VoiceAsk<'static> {
        VoiceAsk::Bye
    }

    fn id(note: &VoiceNote) -> u32 {
        note.id()
    }

    fn write<W: Write + ?Sized>(ask: &VoiceAsk<'static>, w: &mut W) -> Result<(), IpcError> {
        ask.write(w)
    }

    fn read<R: Read + ?Sized>(r: &mut R) -> Result<VoiceNote, IpcError> {
        VoiceNote::read(r)
    }
}

impl Streams for VoiceFrames {
    fn ask_id(ask: &VoiceAsk<'static>) -> u32 {
        ask.id()
    }

    /// A run's end, an ask's answer, the held recording's last piece, or a fault other than a card's.
    fn last(note: &VoiceNote) -> bool {
        match note {
            VoiceNote::Done { .. }
            | VoiceNote::Devices { .. }
            | VoiceNote::Facts { .. }
            | VoiceNote::Checked { .. } => true,
            VoiceNote::Fault { fault, .. } => SpeechError::from(*fault).ends(),
            VoiceNote::Sound { piece, .. } => piece.last,
            VoiceNote::Level { .. }
            | VoiceNote::Heard { .. }
            | VoiceNote::Stage { .. }
            | VoiceNote::Warm { .. }
            | VoiceNote::Piece { .. } => false,
        }
    }
}

/// The fetch job's frames (ADR-0042 decision 2): one download, its pause, resume and cancel, and its progress.
#[derive(Debug)]
pub struct FetchFrames;

impl Frames for FetchFrames {
    const JOB: &'static str = FETCH_JOB;
    type Ask = FetchAsk<'static>;
    type Reply = FetchNote;

    fn bye() -> FetchAsk<'static> {
        FetchAsk::Bye
    }

    fn id(note: &FetchNote) -> u32 {
        note.id()
    }

    fn write<W: Write + ?Sized>(ask: &FetchAsk<'static>, w: &mut W) -> Result<(), IpcError> {
        ask.write(w)
    }

    fn read<R: Read + ?Sized>(r: &mut R) -> Result<FetchNote, IpcError> {
        FetchNote::read(r)
    }
}

impl Streams for FetchFrames {
    fn ask_id(ask: &FetchAsk<'static>) -> u32 {
        ask.id()
    }

    /// Every download ends with its file named or one fault; progress goes on.
    fn last(note: &FetchNote) -> bool {
        match note {
            FetchNote::Done { .. } | FetchNote::Fault { .. } => true,
            FetchNote::Progress { .. } => false,
        }
    }
}

#[cfg(test)]
mod tests;
