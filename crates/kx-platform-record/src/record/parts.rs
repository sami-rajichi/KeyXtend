//! What a run and a copy are made of: the clock, the screen, the sound sources and the file being written, each the
//! system's own or a fake, so the timeline is tested without a screen, a device or a sleep.

use std::any::Any;
use std::path::Path;
use std::time::Instant;

use super::{Plan, RecordFault, Switch};

/// A video's frame size and rate.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Shape {
    /// Width in pixels, even and above 0.
    pub width: u32,
    /// Height in pixels, even and above 0.
    pub height: u32,
    /// Frames a second, above 0.
    pub fps: u32,
}

/// Sound a source heard: stereo frames at the track's rate, interleaved, and when its first frame was heard.
#[derive(Clone, Debug, PartialEq)]
pub struct Packet {
    /// When its first frame was captured.
    pub at: Instant,
    /// Left and right samples, in turn, from -1 to 1.
    pub samples: Vec<f32>,
}

/// The time a run paces its frames by.
pub trait Clock {
    /// Now.
    fn now(&self) -> Instant;

    /// Waits until `at`; at once when it has passed.
    ///
    /// # Errors
    /// `Failed` when the wait cannot be made.
    fn until(&mut self, at: Instant) -> Result<(), RecordFault>;
}

/// The part being filmed.
pub trait Screen<F> {
    /// The newest frame since the last call, or `None` when none came.
    ///
    /// # Errors
    /// `Failed` when the capture broke.
    fn grab(&mut self) -> Result<Option<F>, RecordFault>;
}

/// A sound source, heard until dropped.
pub trait Ear {
    /// The packets heard since the last call, oldest first.
    fn heard(&mut self) -> Vec<Packet>;

    /// True once its stream stopped for good; a gap it went on through is no failure.
    fn failed(&self) -> bool;
}

/// A video file being written.
pub trait Sink<F> {
    /// Writes `frame` as frame `slot`, which plays from `slot / fps` s.
    ///
    /// # Errors
    /// `Full` when the drive filled up, else `Failed`.
    fn frame(&mut self, slot: u64, frame: &F) -> Result<(), RecordFault>;

    /// Writes stereo `pcm` from sound frame `first` on.
    ///
    /// # Errors
    /// As `frame`.
    fn sound(&mut self, first: u64, pcm: &[i16]) -> Result<(), RecordFault>;

    /// Ends the file, whole and playable.
    ///
    /// # Errors
    /// As `frame`.
    fn finish(self: Box<Self>) -> Result<(), RecordFault>;
}

/// One thing a video being read gives, in order.
pub enum Piece<S> {
    /// A decoded frame, and the slot it plays at, at the video's rate.
    Frame(u64, S),
    /// Stereo 16-bit sound at the track's rate.
    Sound(Vec<i16>),
}

/// A video being read for a re-timed copy.
pub trait Reel<S> {
    /// Its frame size and rate.
    fn shape(&self) -> Shape;

    /// Its length in ms.
    fn length_ms(&self) -> u32;

    /// The next frame or sound, `None` at the end.
    ///
    /// # Errors
    /// `Failed` when the file cannot be read.
    fn next(&mut self) -> Result<Option<Piece<S>>, RecordFault>;
}

/// A run's capture, file and clock, opened together.
pub struct Film<F> {
    /// The part being filmed.
    pub screen: Box<dyn Screen<F>>,
    /// The video being written.
    pub sink: Box<dyn Sink<F>>,
    /// The clock its frames keep to.
    pub clock: Box<dyn Clock>,
}

/// The parts a recording and a copy are made of: the system's own, or fakes in tests.
pub trait Studio: Send + Sync + 'static {
    /// A captured frame.
    type Frame;
    /// A decoded frame of a video being re-timed.
    type Still;

    /// Readies this thread for a video job (DPI, COM and Media Foundation); the guard holds them until dropped.
    ///
    /// # Errors
    /// `NoVideo` without Media Foundation (edge 14), else `Failed`.
    fn ready(&self) -> Result<Box<dyn Any>, RecordFault>;

    /// The room left on `folder`'s drive, in MiB; `None` when it cannot be read.
    fn room_mb(&self, folder: &Path) -> Option<u64>;

    /// Opens a sound source.
    ///
    /// # Errors
    /// `NoMic` or `MicRefused` for the microphone, `NoSound` for the PC's sound (edge 11).
    fn ear(&self, which: Switch) -> Result<Box<dyn Ear>, RecordFault>;

    /// Opens the capture of `plan`'s part, its video file and its clock.
    ///
    /// # Errors
    /// `NoVideo` without an encoder, `Full` without room, else `Failed`.
    fn film(&self, plan: &Plan) -> Result<Film<Self::Frame>, RecordFault>;

    /// Opens `from` to re-time.
    ///
    /// # Errors
    /// `Failed` when it cannot be read.
    fn reel(&self, from: &Path) -> Result<Box<dyn Reel<Self::Still>>, RecordFault>;

    /// Opens a re-timed copy of `shape` at `to`.
    ///
    /// # Errors
    /// As `film`.
    fn cut(&self, to: &Path, shape: Shape) -> Result<Box<dyn Sink<Self::Still>>, RecordFault>;
}
