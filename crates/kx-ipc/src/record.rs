//! The record job's frames (ADR-0056 decisions 3 to 5): the host starts a recording of a part of the screen into a
//! video of the Snips folder's `unsaved` part, switches its sounds, stops or cancels it, or asks for a re-timed copy;
//! the job tells the time, each switch made, the end with the video's length, or a fixed fault over the streaming
//! session (ADR-0042).

pub(crate) mod ask;
mod body;
pub(crate) mod note;
pub(crate) mod wire;

use crate::code::Code;

/// A part of the screens in physical desktop pixels: never empty, and its right and bottom edges stay in the pixel
/// range.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Area {
    /// Left edge.
    pub x: i32,
    /// Top edge.
    pub y: i32,
    /// Width, 1 at least.
    pub w: i32,
    /// Height, 1 at least.
    pub h: i32,
}

/// Why a video ended whole and kept.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RecordEnd {
    /// Stop or goodbye, or a re-timed copy is done.
    Stopped = 1,
    /// The longest length, `max_min`, was reached (edge 9).
    Longest = 2,
    /// Its drive was about to drop under `min_free_mb` (edge 8).
    Full = 3,
}

impl Code for RecordEnd {
    const ALL: &'static [Self] = &[Self::Stopped, Self::Longest, Self::Full];

    fn code(self) -> u8 {
        self as u8
    }
}

/// A sound a run switches while it records (edge 12).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Switch {
    /// The microphone.
    Mic = 1,
    /// The computer's sound.
    Sound = 2,
}

impl Code for Switch {
    const ALL: &'static [Self] = &[Self::Mic, Self::Sound];

    fn code(self) -> u8 {
        self as u8
    }
}

/// Why a recording, a switch or a copy failed; a fixed code, never a library's words. Before a run's first `Time`, any
/// fault ends its start; after it, the job answers each switch with `Switched` or that sound's fault and the run goes
/// on, while any other fault ends the run.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RecordFault {
    /// No microphone was found (edge 11).
    NoMic = 1,
    /// The system's privacy switch refused the microphone (edge 11).
    MicRefused = 2,
    /// No sound device was found for the computer's sound (edge 11).
    NoSound = 3,
    /// The system has no Media Foundation, so no video encoder (edge 14).
    NoVideo = 4,
    /// The drive holds less than `min_free_mb`, or filled up while a copy was written.
    Full = 5,
    /// The job or the system failed inside; a run's or a copy's broken video is deleted (edge 13). The host deletes it
    /// when it is still there, as after a delete that failed.
    Failed = 6,
}

impl Code for RecordFault {
    const ALL: &'static [Self] = &[
        Self::NoMic,
        Self::MicRefused,
        Self::NoSound,
        Self::NoVideo,
        Self::Full,
        Self::Failed,
    ];

    fn code(self) -> u8 {
        self as u8
    }
}
