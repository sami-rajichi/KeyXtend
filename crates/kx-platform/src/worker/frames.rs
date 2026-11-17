//! Each job's frames from `kx-ipc`: the media and vault jobs' asks and replies, and the voice, fetch and speech-net
//! jobs' asks and the notes they send on their own.

use super::{Frames, Streams};
use kx_ipc::{
    Ask, FETCH_JOB, FetchAsk, FetchNote, IpcError, MEDIA_JOB, NET_JOB, NetAsk, NetNote, Reply,
    Stage, VAULT_JOB, VOICE_JOB, VaultAsk, VaultReply, VoiceAsk, VoiceNote,
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

    /// The microphone closing, after which the job writes the rest on its own, as after Stop.
    fn shift(note: &VoiceNote) -> bool {
        let closed = [Stage::Shut, Stage::Limit, Stage::Silence];
        matches!(note, VoiceNote::Stage { stage, .. } if closed.contains(stage))
    }

    /// A run's end with the job's peak memory, as the job reads none of its own.
    fn stamp(note: &mut VoiceNote, peak: &mut dyn FnMut() -> Option<u32>) {
        if let VoiceNote::Done { memory_mb, .. } = note {
            *memory_mb = (*memory_mb).max(peak().unwrap_or_default());
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

/// The speech-net job's frames (ADR-0047): a run's pieces or a key's test, and the texts and ends it sends.
#[derive(Debug)]
pub struct NetFrames;

impl Frames for NetFrames {
    const JOB: &'static str = NET_JOB;
    type Ask = NetAsk<'static>;
    type Reply = NetNote;

    fn bye() -> NetAsk<'static> {
        NetAsk::Bye
    }

    fn id(note: &NetNote) -> u32 {
        note.id()
    }

    fn write<W: Write + ?Sized>(ask: &NetAsk<'static>, w: &mut W) -> Result<(), IpcError> {
        ask.write(w)
    }

    fn read<R: Read + ?Sized>(r: &mut R) -> Result<NetNote, IpcError> {
        NetNote::read(r)
    }
}

impl Streams for NetFrames {
    fn ask_id(ask: &NetAsk<'static>) -> u32 {
        ask.id()
    }

    /// A run's or a test's end, or its fault; a text goes on.
    fn last(note: &NetNote) -> bool {
        match note {
            NetNote::Done { .. } | NetNote::Fault { .. } => true,
            NetNote::Text { .. } => false,
        }
    }
}

#[cfg(test)]
mod tests;
