//! The screen recorder (ADR-0056 decisions 3 to 5): one run films a part at `fps`, mixes the microphone and the PC's
//! sound into one track in step with the picture, obeys switches, Stop and Cancel between frames, and ends itself at
//! the longest length or the free-space floor. It runs over a `Studio`'s parts: the system's, or fakes in tests.

mod lane;
mod mix;
mod parts;
mod run;
pub mod stereo;
mod take;

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::table::Table;
use kx_ipc::MAX_VIDEO_MS;
pub use kx_ipc::{Area, RecordEnd, RecordFault, RecordNote, Switch};
pub use parts::{Clock, Ear, Film, Packet, Piece, Reel, Screen, Shape, Sink, Studio};
pub use run::record;

/// The sound track's rate, one the system's AAC encoder takes.
pub const RATE: u32 = 48_000;
/// The sound track's channels: stereo.
pub const CHANNELS: u16 = 2;
/// Milliseconds in a second, the unit of the notes' times.
const MS_S: u64 = 1_000;
/// Seconds in a minute, the unit of `max_min`.
const MIN_S: u64 = 60;

/// One recording, as the start ask gives it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    /// The run's id, which every note repeats.
    pub id: u32,
    /// The part filmed, in physical pixels.
    pub area: Area,
    /// Frames a second.
    pub fps: u8,
    /// The microphone records from the start.
    pub mic: bool,
    /// The PC's sound records from the start.
    pub sound: bool,
    /// The longest video, in minutes.
    pub max_min: u16,
    /// The MiB its drive keeps free.
    pub min_free_mb: u32,
    /// The new video; never one that exists.
    pub file: PathBuf,
}

impl Plan {
    /// The video's frame size and rate: the part's sizes made even, as H.264 wants.
    #[must_use]
    pub fn shape(&self) -> Shape {
        let even = |n: i32| u32::try_from(n - n % 2).unwrap_or_default();
        Shape {
            width: even(self.area.w),
            height: even(self.area.h),
            fps: u32::from(self.fps),
        }
    }

    /// The slots the longest video holds.
    fn cap(&self) -> u64 {
        u64::from(self.max_min) * MIN_S * u64::from(self.fps)
    }
}

/// Deletes `file`, a video that must not be kept; false when it is still there, as when the system still holds it.
#[must_use]
pub fn scrap(file: &Path) -> bool {
    match fs::remove_file(file) {
        Ok(()) => true,
        Err(e) => e.kind() == ErrorKind::NotFound,
    }
}

/// A time or length in ms as a note tells it: never past `MAX_VIDEO_MS`, which the frames refuse.
pub(crate) fn told_ms(ms: u64) -> u32 {
    u32::try_from(ms).unwrap_or(u32::MAX).min(MAX_VIDEO_MS)
}

/// The most sound frames a source keeps waiting and one write holds, so a long stall never takes its gap at once.
pub(crate) fn queue_frames(table: &Table) -> usize {
    usize::try_from(mix::frames(table.sound.queue_ms))
        .unwrap_or(0)
        .max(1)
}

/// What the job asks a running recording, between frames.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Order {
    /// Switch a sound on or off; the run answers with its ack or that sound's fault.
    Switch(Switch, bool),
    /// Stop and keep the video.
    Stop,
    /// Stop and delete the video, telling nothing more, or `Failed` when it could not be deleted.
    Cancel,
}

#[cfg(test)]
pub(crate) mod tests;
