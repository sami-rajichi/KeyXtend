//! The bodies both voice sides send: names, paths, a model and its target, a setup, a sound piece and a device. Each
//! field's length is checked first, then its content and range.

use super::wire::{DEVICES, NAME, PATH, SILENCE_OFF, SOUND, TAG};
use super::{Device, DeviceKind, Model, Setup, SoundPiece, Target};
use crate::code::Code;
use crate::error::IpcError;
use crate::limits::{
    LEVEL_HZ, PAD_MS, PIECE_S, RECORD_MAX_S, SAMPLE_BYTES, SILENCE_STOP_S, THREADS,
};
use crate::rules::{self, ok_if, within};
use crate::wiped::Wiped;
use crate::wire::{Payload, Sink};
use std::borrow::Cow;
use std::io::Read;

/// Sends a short ASCII name.
pub(crate) fn put_tag(s: &mut dyn Sink, text: &str) -> Result<(), IpcError> {
    s.field(TAG, text.as_bytes())?;
    rules::tag(text)
}

/// Reads a short ASCII name.
pub(crate) fn read_tag<R: Read + ?Sized>(p: &mut Payload<'_, R>) -> Result<String, IpcError> {
    let text = p.text(TAG)?;
    rules::tag(&text)?;
    Ok(text)
}

/// Sends a file path.
pub(crate) fn put_path(s: &mut dyn Sink, text: &str) -> Result<(), IpcError> {
    s.field(PATH, text.as_bytes())?;
    rules::path(text)
}

/// Reads a file path.
pub(crate) fn read_path<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
) -> Result<Cow<'static, str>, IpcError> {
    let text = p.text(PATH)?;
    rules::path(&text)?;
    Ok(Cow::Owned(text))
}

/// Sends a shown name.
fn put_name(s: &mut dyn Sink, text: &str) -> Result<(), IpcError> {
    s.field(NAME, text.as_bytes())?;
    rules::name(text)
}

/// Reads a shown name.
fn read_name<R: Read + ?Sized>(p: &mut Payload<'_, R>) -> Result<String, IpcError> {
    let text = p.text(NAME)?;
    rules::name(&text)?;
    Ok(text)
}

/// Sends a model: its path, language, runner, target and threads.
pub(crate) fn put_model(s: &mut dyn Sink, m: &Model<'_>) -> Result<(), IpcError> {
    put_path(s, &m.path)?;
    put_tag(s, &m.lang)?;
    put_tag(s, &m.runner)?;
    match &m.target {
        Target::Cpu => s.byte(DeviceKind::Cpu.code())?,
        Target::Card { id, need_mb } => {
            s.byte(DeviceKind::Card.code())?;
            put_tag(s, id)?;
            s.u32(*need_mb)?;
        }
    }
    s.byte(within(m.threads, &THREADS)?)
}

/// Reads a model.
pub(crate) fn read_model<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
) -> Result<Model<'static>, IpcError> {
    let path = read_path(p)?;
    let lang = Cow::Owned(read_tag(p)?);
    let runner = Cow::Owned(read_tag(p)?);
    let target = match DeviceKind::from_code(p.byte()?) {
        Some(DeviceKind::Cpu) => Target::Cpu,
        Some(DeviceKind::Card) => {
            let id = Cow::Owned(read_tag(p)?);
            Target::Card {
                id,
                need_mb: p.u32()?,
            }
        }
        Some(DeviceKind::BuiltIn) | None => return Err(IpcError::BadField),
    };
    let threads = within(p.byte()?, &THREADS)?;
    Ok(Model {
        path,
        lang,
        runner,
        target,
        threads,
    })
}

/// Sends a setup: whether it names a model, the model, the settings, then the detector's path.
pub(crate) fn put_setup(s: &mut dyn Sink, setup: &Setup<'_>) -> Result<(), IpcError> {
    s.byte(u8::from(setup.model.is_some()))?;
    if let Some(model) = &setup.model {
        put_model(s, model)?;
    }
    s.u16(within(setup.pad_ms, &PAD_MS)?)?;
    s.byte(within(setup.piece_s, &PIECE_S)?)?;
    s.u16(within(setup.record_max_s, &RECORD_MAX_S)?)?;
    s.byte(within(setup.level_hz, &LEVEL_HZ)?)?;
    let silence = match setup.silence_stop_s {
        Some(n) => within(n, &SILENCE_STOP_S)?,
        None => SILENCE_OFF,
    };
    s.byte(silence)?;
    put_path(s, &setup.detector)
}

/// Reads a setup.
pub(crate) fn read_setup<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
) -> Result<Setup<'static>, IpcError> {
    let model = if p.flag()? {
        Some(read_model(p)?)
    } else {
        None
    };
    let pad_ms = within(p.u16()?, &PAD_MS)?;
    let piece_s = within(p.byte()?, &PIECE_S)?;
    let record_max_s = within(p.u16()?, &RECORD_MAX_S)?;
    let level_hz = within(p.byte()?, &LEVEL_HZ)?;
    let silence_stop_s = match p.byte()? {
        SILENCE_OFF => None,
        n => Some(within(n, &SILENCE_STOP_S)?),
    };
    Ok(Setup {
        model,
        pad_ms,
        piece_s,
        record_max_s,
        level_hz,
        silence_stop_s,
        detector: read_path(p)?,
    })
}

/// `index` when it is below `most`.
fn sound_index(index: u16, most: usize) -> Result<u16, IpcError> {
    ok_if(usize::from(index) < most)?;
    Ok(index)
}

/// Sends a sound piece: its index below `most`, whether it is the last, then its whole samples.
pub(crate) fn put_sound(
    s: &mut dyn Sink,
    piece: &SoundPiece<'_>,
    most: usize,
) -> Result<(), IpcError> {
    s.u16(sound_index(piece.index, most)?)?;
    s.byte(u8::from(piece.last))?;
    s.field(SOUND, &piece.bytes)?;
    ok_if(piece.bytes.len().is_multiple_of(SAMPLE_BYTES))
}

/// Reads a sound piece with its index below `most` into a wiped buffer, refusing a part sample before its bytes.
pub(crate) fn read_sound<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
    most: usize,
) -> Result<SoundPiece<'static>, IpcError> {
    let index = sound_index(p.u16()?, most)?;
    let last = p.flag()?;
    let n = p.count(SOUND)?;
    ok_if(n.is_multiple_of(SAMPLE_BYTES))?;
    let bytes = Wiped::Kept(p.wiped(n)?);
    Ok(SoundPiece { index, last, bytes })
}

/// Sends a device list: its count, then each device's kind, id, name, memory and driver.
pub(crate) fn put_devices(s: &mut dyn Sink, list: &[Device]) -> Result<(), IpcError> {
    s.count(DEVICES, list.len())?;
    for d in list {
        s.byte(d.kind.code())?;
        put_tag(s, &d.id)?;
        put_name(s, &d.name)?;
        ok_if(d.free_mb <= d.total_mb)?;
        s.u32(d.total_mb)?;
        s.u32(d.free_mb)?;
        put_name(s, &d.driver)?;
    }
    Ok(())
}

/// Reads a device list.
pub(crate) fn read_devices<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
) -> Result<Vec<Device>, IpcError> {
    let count = p.count(DEVICES)?;
    let mut list = Vec::new();
    for _ in 0..count {
        let kind = DeviceKind::from_code(p.byte()?).ok_or(IpcError::BadField)?;
        let (id, name) = (read_tag(p)?, read_name(p)?);
        let (total_mb, free_mb) = (p.u32()?, p.u32()?);
        ok_if(free_mb <= total_mb)?;
        let driver = read_name(p)?;
        list.push(Device {
            kind,
            id,
            name,
            total_mb,
            free_mb,
            driver,
        });
    }
    Ok(list)
}
