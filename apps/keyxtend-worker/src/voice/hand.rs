//! The held recording and its hand-over: `Sound` notes of whole 16-bit little-endian samples, each at most
//! `MAX_SOUND_BYTES`, a listen-only run's packed speech as `Speech` notes at Stop, and the `Sound` asks a recording
//! moved from another job comes back in, checked piece by piece.

use super::Notes;
use super::job::Fail;
use super::wipe::Buf;
use kx_ipc::{
    MAX_RECORD_BYTES, MAX_SOUND_BYTES, MAX_SPEECH_PIECES, SAMPLE_BYTES, SoundPiece, VoiceFault,
    VoiceNote, Wiped,
};
use std::ops::Range;
use zeroize::Zeroizing;

/// Samples in one handed-over piece.
pub const PIECE_SAMPLES: usize = MAX_SOUND_BYTES / SAMPLE_BYTES;
/// Samples in the longest recording.
const MOST_SAMPLES: usize = MAX_RECORD_BYTES / SAMPLE_BYTES;

/// A recording kept until Close, with the run that made or took it and the last run that wrote it again.
pub struct Held {
    sound: Buf<i16>,
    ids: [u32; 2],
}

impl Held {
    /// The recording `sound` of run `id`.
    pub fn new(id: u32, sound: Buf<i16>) -> Self {
        Self {
            sound,
            ids: [id, id],
        }
    }

    /// True when run `id` made it or last wrote it again.
    pub fn owns(&self, id: u32) -> bool {
        self.ids.contains(&id)
    }

    /// Notes that run `id` writes it again.
    pub fn serve(&mut self, id: u32) {
        self.ids[1] = id;
    }

    /// Its samples.
    pub fn sound(&self) -> &[i16] {
        &self.sound
    }
}

/// Hands `sound` over as run `id`'s `Sound` notes in order, the last marked; an empty one is one empty last piece.
///
/// # Errors
/// `Out` when the pipe fails, `Memory` when a piece's buffer cannot be had.
pub fn over(id: u32, sound: &[i16], out: &mut dyn Notes) -> Result<(), Fail> {
    let count = sound.len().div_ceil(PIECE_SAMPLES).max(1);
    for (k, part) in sound
        .chunks(PIECE_SAMPLES)
        .chain(sound.is_empty().then_some(&[][..]))
        .enumerate()
    {
        let index = u16::try_from(k).map_err(|_| VoiceFault::Internal)?;
        let bytes = Wiped::Kept(bytes(&[part])?);
        let piece = SoundPiece {
            index,
            last: k + 1 == count,
            bytes,
        };
        out.tell(VoiceNote::Sound { id, piece })?;
    }
    Ok(())
}

/// Hands a listen-only run's speech over as run `id`'s `Speech` notes in order, each piece the stretches of `sound` it
/// joins, the last marked; no speech is no note.
///
/// # Errors
/// `Out` when the pipe fails, `Memory` when a piece's buffer cannot be had, `Internal` for a stretch past `sound` or a
/// piece past `MAX_SPEECH_PIECES`.
pub fn speech(
    id: u32,
    sound: &[i16],
    pieces: &[Vec<Range<usize>>],
    out: &mut dyn Notes,
) -> Result<(), Fail> {
    for (k, spans) in pieces.iter().enumerate() {
        let index = u16::try_from(k).ok().filter(|_| k < MAX_SPEECH_PIECES);
        let index = index.ok_or(VoiceFault::Internal)?;
        let parts: Option<Vec<&[i16]>> = spans.iter().map(|s| sound.get(s.clone())).collect();
        let bytes = Wiped::Kept(bytes(&parts.ok_or(VoiceFault::Internal)?)?);
        let last = k + 1 == pieces.len();
        let piece = SoundPiece { index, last, bytes };
        out.tell(VoiceNote::Speech { id, piece })?;
    }
    Ok(())
}

/// `parts` joined as little-endian bytes in a wiped buffer, made in one allocation.
fn bytes(parts: &[&[i16]]) -> Result<Zeroizing<Vec<u8>>, VoiceFault> {
    let mut out = Zeroizing::new(Vec::new());
    let len: usize = parts.iter().map(|p| p.len()).sum();
    out.try_reserve_exact(len * SAMPLE_BYTES)
        .map_err(|_| VoiceFault::Memory)?;
    for s in parts.iter().flat_map(|p| p.iter()) {
        out.extend_from_slice(&s.to_le_bytes());
    }
    Ok(out)
}

/// A recording coming in from the host, piece by piece.
pub struct HandIn {
    id: u32,
    next: u16,
    sound: Buf<i16>,
}

impl HandIn {
    /// An empty hand-in for run `id`, whose room never doubles past the longest recording.
    pub fn new(id: u32) -> Self {
        Self {
            id,
            next: 0,
            sound: Buf::with_limit(MOST_SAMPLES),
        }
    }

    /// The run it comes for.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Adds `piece`; true when it was the last.
    ///
    /// # Errors
    /// `OutOfStep` for a piece out of order or past the longest recording, `Memory` when the buffer cannot grow.
    pub fn take(&mut self, piece: &SoundPiece<'_>) -> Result<bool, VoiceFault> {
        let samples = piece.bytes.len() / SAMPLE_BYTES;
        if piece.index != self.next || self.sound.len() + samples > MOST_SAMPLES {
            return Err(VoiceFault::OutOfStep);
        }
        self.sound.reserve(samples)?;
        for b in piece.bytes.as_chunks::<SAMPLE_BYTES>().0 {
            self.sound.push(i16::from_le_bytes(*b))?;
        }
        self.next = self.next.saturating_add(1);
        Ok(piece.last)
    }

    /// The whole recording, held for run `id`.
    pub fn held(self) -> Held {
        Held::new(self.id, self.sound)
    }
}
