//! The record asks' bodies: a part, a rate, a speed and a video's path, each checked by both sides, and a start.

use super::Area;
use super::ask::RecordAsk;
use crate::error::IpcError;
use crate::limits::snip::{MP4_EXT, SNIP_FPS, SNIP_MAX_MIN, SNIP_MIN_FREE_MB, SNIP_SPEEDS};
use crate::pick::{put_path, read_path};
use crate::rules::{ok_if, typed, within};
use crate::wire::{Payload, Sink};
use std::borrow::Cow;
use std::io::Read;

/// Refuses an empty part, or one whose right or bottom edge passes `i32::MAX`.
fn area(a: Area) -> Result<Area, IpcError> {
    let edges = a.x.checked_add(a.w).is_some() && a.y.checked_add(a.h).is_some();
    ok_if(a.w > 0 && a.h > 0 && edges)?;
    Ok(a)
}

/// `fps` when `SNIP_FPS` lists it.
fn fps(fps: u8) -> Result<u8, IpcError> {
    ok_if(SNIP_FPS.contains(&fps))?;
    Ok(fps)
}

/// `times` when `SNIP_SPEEDS` lists it.
pub(super) fn speed(times: u8) -> Result<u8, IpcError> {
    ok_if(SNIP_SPEEDS.contains(&times))?;
    Ok(times)
}

/// Sends a video's path: a plain path whose name ends in `MP4_EXT`.
pub(super) fn put_video(s: &mut dyn Sink, text: &str) -> Result<(), IpcError> {
    put_path(s, text)?;
    typed(text, MP4_EXT)
}

/// Reads a video's path.
pub(super) fn read_video<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
) -> Result<Cow<'static, str>, IpcError> {
    let text = read_path(p)?;
    typed(&text, MP4_EXT)?;
    Ok(text)
}

/// Sends a start: its part, rate, switches, longest length and floor, then its video's path.
pub(super) fn put_start(s: &mut dyn Sink, ask: &RecordAsk<'_>) -> Result<(), IpcError> {
    let RecordAsk::Start {
        area: a,
        fps: rate,
        mic,
        sound,
        max_min,
        min_free_mb,
        file,
        ..
    } = ask
    else {
        return Ok(());
    };
    let a = area(*a)?;
    [a.x, a.y, a.w, a.h]
        .into_iter()
        .try_for_each(|n| s.i32(n))?;
    s.byte(fps(*rate)?)?;
    s.byte(u8::from(*mic))?;
    s.byte(u8::from(*sound))?;
    s.u16(within(*max_min, &SNIP_MAX_MIN)?)?;
    s.u32(within(*min_free_mb, &SNIP_MIN_FREE_MB)?)?;
    put_video(s, file)
}

/// Reads a part's four edges, then checks them.
fn read_area<R: Read + ?Sized>(p: &mut Payload<'_, R>) -> Result<Area, IpcError> {
    let mut edge = || p.i32();
    area(Area {
        x: edge()?,
        y: edge()?,
        w: edge()?,
        h: edge()?,
    })
}

/// Reads a start of run `id`.
pub(super) fn read_start<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
    id: u32,
) -> Result<RecordAsk<'static>, IpcError> {
    let area = read_area(p)?;
    let fps = fps(p.byte()?)?;
    let (mic, sound) = (p.flag()?, p.flag()?);
    let max_min = within(p.u16()?, &SNIP_MAX_MIN)?;
    let min_free_mb = within(p.u32()?, &SNIP_MIN_FREE_MB)?;
    let file = read_video(p)?;
    Ok(RecordAsk::Start {
        id,
        area,
        fps,
        mic,
        sound,
        max_min,
        min_free_mb,
        file,
    })
}
