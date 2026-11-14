//! The sound source: the default microphone through `cpal` on Windows, or a fake in tests. It hands each chunk to the
//! job's feed on its own thread, and dropping the open source closes the microphone.

#[cfg(windows)]
mod live;

use super::Feed;
use kx_ipc::VoiceFault;
use std::any::Any;

/// The sound's format as the source gives it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Format {
    /// Frames a second.
    pub rate: u32,
    /// Interleaved channels in each frame.
    pub channels: u16,
}

/// An open source: its format, and what keeps it open until dropped.
pub struct Opened {
    /// The format its chunks come in.
    pub format: Format,
    /// Dropping it closes the source.
    pub live: Box<dyn Any>,
}

/// A source of sound for a run.
pub trait Source {
    /// Opens it; each chunk of interleaved samples goes to `feed`, on any thread, until `Opened::live` drops.
    ///
    /// # Errors
    /// `NoMic`, `MicBusy` or `MicRefused` when the microphone cannot open, `Internal` for a format it cannot read.
    fn open(&mut self, feed: Feed) -> Result<Opened, VoiceFault>;
}

/// The default microphone.
pub struct Mic;

#[cfg(windows)]
impl Source for Mic {
    fn open(&mut self, feed: Feed) -> Result<Opened, VoiceFault> {
        live::open(feed)
    }
}

/// Other systems get their microphone in P15; until then none is found.
#[cfg(not(windows))]
impl Source for Mic {
    fn open(&mut self, _: Feed) -> Result<Opened, VoiceFault> {
        Err(VoiceFault::NoMic)
    }
}
