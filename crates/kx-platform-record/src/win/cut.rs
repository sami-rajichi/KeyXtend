//! A re-timed copy's file: each kept frame's picture is copied out of its decoded NV12 buffer, rows past the picture
//! left behind and never past the buffer's own end, and written through the Sink Writer with the thinned sound.

use std::path::Path;

use windows::Win32::Media::MediaFoundation::{
    IMF2DBuffer2, MF2DBuffer_LockFlags_Read, MFVideoFormat_NV12,
};
use windows::core::Interface;

use super::reel::Decoded;
use super::refused;
use super::writer::{Writer, memory};
use crate::record::{RecordFault, Shape, Sink};
use crate::table::Table;

/// NV12 keeps one colour row for every two picture rows, and three bytes for every two pixels.
const HALF: usize = 2;
const PAIR_BYTES: usize = 3;

/// A copy being written.
pub(super) struct Cut {
    writer: Writer,
    shape: Shape,
}

impl Cut {
    /// Starts writing `to` as `shape` says.
    pub(super) fn open(to: &Path, shape: Shape, table: &Table) -> Result<Self, RecordFault> {
        let writer = Writer::open(to, &shape, MFVideoFormat_NV12, None, table)?;
        Ok(Self { writer, shape })
    }
}

/// The picture of `frame` at `shape`, its brightness rows then its colour rows, with no row past the picture; short
/// when the buffer holds fewer rows than its frame size says.
#[allow(unsafe_code, reason = "Reads a decoded buffer once.")]
pub(super) fn picture(frame: &Decoded, shape: Shape) -> windows::core::Result<Vec<u8>> {
    let rows = to_usize(frame.size.1);
    // SAFETY: a live sample's own buffer.
    let buffer = unsafe { frame.sample.GetBufferByIndex(0)? };
    let Ok(flat) = buffer.cast::<IMF2DBuffer2>() else {
        return joined(frame, rows, shape);
    };
    let (mut first, mut pitch, mut start, mut len) =
        (std::ptr::null_mut(), 0_i32, std::ptr::null_mut(), 0_u32);
    // SAFETY: a live 2D buffer, locked once and unlocked once; the span starts at its first row and ends at the end of
    // the buffer the lock gives, whatever its frame size says.
    unsafe {
        flat.Lock2DSize(
            MF2DBuffer_LockFlags_Read,
            &raw mut first,
            &raw mut pitch,
            &raw mut start,
            &raw mut len,
        )?;
        let held = (start.addr() + usize::try_from(len).unwrap_or(0)).saturating_sub(first.addr());
        let pitch = usize::try_from(pitch).unwrap_or(0);
        let bytes = std::slice::from_raw_parts(first, span(pitch, rows, held));
        let out = cropped(bytes, pitch, rows, shape);
        flat.Unlock2D()?;
        Ok(out)
    }
}

/// `picture` for a frame whose buffer is not a 2D one: its buffers joined, rows at the frame's width.
#[allow(unsafe_code, reason = "Reads a decoded buffer once.")]
fn joined(frame: &Decoded, rows: usize, shape: Shape) -> windows::core::Result<Vec<u8>> {
    // SAFETY: a live sample's buffers, joined; locked once and unlocked once, the bytes copied out between, never past
    // the length the lock gives.
    unsafe {
        let whole = frame.sample.ConvertToContiguousBuffer()?;
        let (mut first, mut len) = (std::ptr::null_mut(), 0_u32);
        whole.Lock(&raw mut first, None, Some(&raw mut len))?;
        let bytes = std::slice::from_raw_parts(first, usize::try_from(len).unwrap_or(0));
        let out = cropped(bytes, to_usize(frame.size.0), rows, shape);
        whole.Unlock()?;
        Ok(out)
    }
}

/// The bytes an NV12 frame of `rows` brightness rows at `pitch` spans, at most the `held` bytes its buffer holds.
fn span(pitch: usize, rows: usize, held: usize) -> usize {
    pitch.saturating_mul(rows + rows / HALF).min(held)
}

/// The picture's rows of an NV12 frame laid out in `bytes` at `pitch` with `rows` brightness rows, cut to `shape`.
fn cropped(bytes: &[u8], pitch: usize, rows: usize, shape: Shape) -> Vec<u8> {
    let (w, h) = (to_usize(shape.width), to_usize(shape.height));
    let mut out = Vec::with_capacity(w * h * PAIR_BYTES / HALF);
    for row in (0..h).chain((0..h / HALF).map(|r| rows + r)) {
        let start = row * pitch;
        out.extend_from_slice(bytes.get(start..start + w).unwrap_or_default());
    }
    out
}

impl Sink<Decoded> for Cut {
    fn frame(&mut self, slot: u64, frame: &Decoded) -> Result<(), RecordFault> {
        let picture = picture(frame, self.shape).map_err(refused)?;
        let want = to_usize(self.shape.width) * to_usize(self.shape.height) * PAIR_BYTES / HALF;
        if picture.len() != want {
            return Err(RecordFault::Failed);
        }
        self.writer.frame(slot, &memory(&picture).map_err(refused)?)
    }

    fn sound(&mut self, first: u64, pcm: &[i16]) -> Result<(), RecordFault> {
        self.writer.sound(first, pcm)
    }

    fn finish(self: Box<Self>) -> Result<(), RecordFault> {
        self.writer.finish()
    }
}

/// `n` as a count of bytes or rows.
fn to_usize(n: u32) -> usize {
    usize::try_from(n).unwrap_or(0)
}
