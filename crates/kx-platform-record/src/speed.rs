//! Re-timed copies (design "The editor"): at 2× or 4× each frame moves to its slot's half or quarter, one kept a slot,
//! and the sound is thinned by averaging, so picture and sound stay in step; 1× copies the file. A copy always keeps
//! its first frame and sound to its last one, so a very short video still plays (edge 20).

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use kx_ipc::MAX_VIDEO_MS;

use crate::record::{
    CHANNELS, Piece, RATE, RecordFault, Reel, Sink, Studio, queue_frames, scrap, told_ms,
};
use crate::table::table;

/// Milliseconds in a second.
const MS_S: u64 = 1_000;

/// Writes `from` re-timed `times` faster into the new file `to`, and answers the copy's length in ms. A copy that fails
/// is deleted once Media Foundation ended.
///
/// # Errors
/// `NoVideo` without Media Foundation, `Full` when the drive filled up, else `Failed`, as when `to` exists or is
/// `from`, or a failed copy could not be deleted.
pub fn speed<S: Studio>(studio: &S, from: &Path, to: &Path, times: u8) -> Result<u32, RecordFault> {
    if times == 0 || from == to || to.exists() || !from.is_file() {
        return Err(RecordFault::Failed);
    }
    let ready = studio.ready()?;
    let made = copy(studio, from, to, times);
    drop(ready);
    if made.is_err() && !scrap(to) {
        return Err(RecordFault::Failed);
    }
    made
}

/// Copies as `speed` says, leaving a failed copy for the caller to delete.
fn copy<S: Studio>(studio: &S, from: &Path, to: &Path, times: u8) -> Result<u32, RecordFault> {
    let mut reel = studio.reel(from)?;
    if times == 1 {
        fs::copy(from, to).map_err(|e| match e.kind() {
            ErrorKind::StorageFull => RecordFault::Full,
            _ => RecordFault::Failed,
        })?;
        return Ok(told_ms(u64::from(reel.length_ms())));
    }
    let most = queue_frames(table().ok_or(RecordFault::Failed)?);
    let cut = studio.cut(to, reel.shape())?;
    retime(&mut *reel, cut, times, most)
}

/// Moves each frame of `reel` to its slot over `times`, the first one a slot, with the thinned sound, into `cut`; then
/// pads the sound to the last frame's end, `most` frames a write at most.
fn retime<F>(
    reel: &mut dyn Reel<F>,
    mut cut: Box<dyn Sink<F>>,
    times: u8,
    most: usize,
) -> Result<u32, RecordFault> {
    let fps = u64::from(reel.shape().fps.max(1));
    // No frame lands past the longest video, whatever time a broken file gives it.
    let cap = u64::from(MAX_VIDEO_MS) * fps / MS_S;
    // The copy's next free slot, and the sound written.
    let (mut next, mut heard) = (0_u64, 0_u64);
    let mut thin = Thin::new(times);
    while let Some(piece) = reel.next()? {
        match piece {
            Piece::Frame(slot, frame) => {
                let to = slot / u64::from(times);
                if (next..cap).contains(&to) {
                    cut.frame(to, &frame)?;
                    next = to + 1;
                }
            }
            Piece::Sound(pcm) => {
                let pcm = thin.push(&pcm);
                cut.sound(heard, &pcm)?;
                heard += frames(&pcm);
            }
        }
    }
    if next == 0 {
        return Err(RecordFault::Failed);
    }
    pad(&mut *cut, heard, next * u64::from(RATE) / fps, most)?;
    cut.finish()?;
    Ok(told_ms(next * MS_S / fps))
}

/// Writes silence into `cut` from sound frame `from` up to `to`, `most` frames a write at most.
fn pad<F>(cut: &mut dyn Sink<F>, from: u64, to: u64, most: usize) -> Result<(), RecordFault> {
    let mut at = from;
    while at < to {
        let n = usize::try_from(to - at).unwrap_or(usize::MAX).min(most);
        cut.sound(at, &vec![0; n * usize::from(CHANNELS)])?;
        at += u64::try_from(n).unwrap_or(u64::MAX);
    }
    Ok(())
}

/// The stereo frames in `pcm`.
fn frames(pcm: &[i16]) -> u64 {
    u64::try_from(pcm.len() / usize::from(CHANNELS)).unwrap_or(u64::MAX)
}

/// Thins stereo sound `times` over, each output frame the average of `times` input frames, kept across chunks.
pub struct Thin {
    times: i32,
    sum: [i32; 2],
    count: i32,
}

impl Thin {
    /// Thinning `times` over.
    #[must_use]
    pub fn new(times: u8) -> Self {
        Self {
            times: i32::from(times.max(1)),
            sum: [0; 2],
            count: 0,
        }
    }

    /// The thinned frames `pcm`'s frames complete.
    pub fn push(&mut self, pcm: &[i16]) -> Vec<i16> {
        let mut out = Vec::new();
        for frame in pcm.chunks_exact(usize::from(CHANNELS)) {
            for (s, v) in self.sum.iter_mut().zip(frame) {
                *s += i32::from(*v);
            }
            self.count += 1;
            if self.count == self.times {
                let mean = self
                    .sum
                    .map(|s| i16::try_from(s / self.times).unwrap_or_default());
                out.extend(mean);
                (self.sum, self.count) = ([0; 2], 0);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests;
