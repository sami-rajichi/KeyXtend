//! How each voice note's payload is read: the notes a run sends while it goes, and those that answer an ask.

use super::super::body::{read_devices, read_sound, read_tag};
use super::super::wire::{HELD, LANGS, PERCENT, SPOKEN, TEXT};
use super::super::{Stage, VoiceFault};
use super::VoiceNote;
use crate::code::{Code, Kind};
use crate::error::IpcError;
use crate::frame::Head;
use crate::limits::{MAX_PIECES, MAX_SOUND_MS};
use crate::rules::{ok_if, within};
use crate::wire::Payload;
use std::io::Read;

/// Reads a note a run sends while it goes.
pub(super) fn read_run<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
    head: Head,
) -> Result<VoiceNote, IpcError> {
    let id = head.id;
    Ok(match head.kind {
        Kind::Level => VoiceNote::Level {
            id,
            level: p.byte()?,
        },
        Kind::Heard => VoiceNote::Heard {
            id,
            speech: p.flag()?,
        },
        Kind::Stage => {
            let stage = Stage::from_code(p.byte()?).ok_or(IpcError::BadField)?;
            VoiceNote::Stage { id, stage }
        }
        Kind::Warm => VoiceNote::Warm {
            id,
            percent: within(p.byte()?, &PERCENT)?,
        },
        Kind::Piece => read_piece(p, id)?,
        Kind::Done => read_done(p, id)?,
        Kind::VoiceFailed => {
            let fault = VoiceFault::from_code(p.byte()?).ok_or(IpcError::BadField)?;
            VoiceNote::Fault { id, fault }
        }
        Kind::SpeechOut => VoiceNote::Speech {
            id,
            piece: read_sound(p, SPOKEN)?,
        },
        _ => return Err(IpcError::BadKind),
    })
}

/// Reads a piece's index and text.
fn read_piece<R: Read + ?Sized>(p: &mut Payload<'_, R>, id: u32) -> Result<VoiceNote, IpcError> {
    let index = p.u16()?;
    ok_if(usize::from(index) < MAX_PIECES)?;
    Ok(VoiceNote::Piece {
        id,
        index,
        text: p.secret(TEXT)?,
    })
}

/// Reads a done note's fields.
fn read_done<R: Read + ?Sized>(p: &mut Payload<'_, R>, id: u32) -> Result<VoiceNote, IpcError> {
    let pieces = p.u16()?;
    let sound_ms = p.u32()?;
    ok_if(usize::from(pieces) <= MAX_PIECES && sound_ms <= MAX_SOUND_MS)?;
    let (wait_ms, load_ms, memory_mb) = (p.u32()?, p.u32()?, p.u32()?);
    let card = p.flag()?;
    Ok(VoiceNote::Done {
        id,
        pieces,
        sound_ms,
        wait_ms,
        load_ms,
        memory_mb,
        card,
    })
}

/// Reads a note that answers an ask: devices, facts, a check result or a sound piece.
pub(super) fn read_answer<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
    head: Head,
) -> Result<VoiceNote, IpcError> {
    let id = head.id;
    Ok(match head.kind {
        Kind::DeviceList => VoiceNote::Devices {
            id,
            list: read_devices(p)?,
        },
        Kind::FileFacts => {
            let runner = read_tag(p)?;
            let langs = (0..p.count(LANGS)?)
                .map(|_| read_tag(p))
                .collect::<Result<_, _>>()?;
            VoiceNote::Facts { id, runner, langs }
        }
        Kind::CardChecked => {
            let (card_ms, cpu_ms) = (p.u32()?, p.u32()?);
            let (card_text, cpu_text) = (p.secret(TEXT)?, p.secret(TEXT)?);
            VoiceNote::Checked {
                id,
                card_ms,
                cpu_ms,
                card_text,
                cpu_text,
            }
        }
        Kind::SoundOut => VoiceNote::Sound {
            id,
            piece: read_sound(p, HELD)?,
        },
        _ => return Err(IpcError::BadKind),
    })
}
