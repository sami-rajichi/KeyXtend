//! Where a snip stands, as the view reads it: the way picked, the part taken with its strip, the countdown, the
//! recording and the picture editor. Pixels never print.

use kx_module_api::Mono;
use kx_module_api::ports::{BarKey, Frame, Keep, Part, Point, Shot, Speed};

use super::io::Video;
use crate::part::Home;

/// Where a snip stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stage {
    /// No snip runs.
    Idle,
    /// The screens are being frozen.
    Freezing,
    /// The capture bar over the frozen screens.
    Picking(Picking),
    /// A part taken, with the strip.
    Strip(Strip),
    /// The countdown before a video, on the live screen.
    Count(Count),
    /// A video records.
    Record(Rec),
    /// The picture editor; the keyboard is back and the Snip key unlit.
    Edit(Edit),
}

/// Picking: the way, and a region's first corner while the second is awaited.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Picking {
    /// One of the five ways of the capture bar.
    pub way: BarKey,
    /// A region's first corner.
    pub corner: Option<Point>,
}

/// A recording's sound switches.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Sounds {
    /// The microphone.
    pub mic: bool,
    /// What the computer plays.
    pub sound: bool,
}

/// A recording kept by its run.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Recorded {
    /// Its video.
    pub video: Video,
    /// Its length, in ms.
    pub length_ms: u32,
}

/// What a strip holds; only a picture holds pixels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Held {
    /// A picture, its pixels as frozen: Copy, Edit, Save and Close.
    Picture(Shot),
    /// A video's part before it records: Record, Mic, Sound and Close.
    Ready(Sounds),
    /// A recorded video at its speed: its length, then Play, Speed, Copy, Save and Close (ADR-0060).
    Video(Reel),
}

/// A part taken, with the strip beside it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Strip {
    /// The part.
    pub part: Part,
    /// The screen holding it, which takes the strip.
    pub home: Home,
    /// What the strip holds.
    pub held: Held,
    /// What was just done, shown with a tick before the snip ends.
    pub done: Option<Keep>,
    /// A region's frame on the frozen screens; `None` for a window, a screen or a recorded video.
    pub frame: Option<Frame>,
}

impl Strip {
    /// True while a frame's handle is held, so the strip hides and its keys cannot act.
    #[must_use]
    pub fn gripped(&self) -> bool {
        self.frame.is_some_and(|f| f.grip.is_some())
    }

    /// Its picture as frozen; `None` for a video, which holds no pixels.
    #[must_use]
    pub fn picture(&self) -> Option<&Shot> {
        match &self.held {
            Held::Picture(shot) => Some(shot),
            Held::Ready(_) | Held::Video(_) => None,
        }
    }
}

/// A video's part and its sound switches, from Record to its end; it holds no frozen pixels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Film {
    /// The part filmed.
    pub part: Part,
    /// The screen holding it, which takes the record bar.
    pub home: Home,
    /// The sound switches.
    pub sounds: Sounds,
}

/// The countdown before a video.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Count {
    /// The video to record.
    pub film: Film,
    /// The seconds left.
    pub left_s: u8,
    /// When the next second goes.
    pub next: Mono,
}

/// A video recording.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rec {
    /// The video filmed.
    pub film: Film,
    /// The video it records into.
    pub video: Video,
    /// How long it has recorded, as last told, in ms.
    pub ms: u32,
    /// Its run told that it started.
    pub live: bool,
    /// Stop was asked; its end is awaited.
    pub stopping: bool,
}

/// A recorded video on its strip, at the speed picked.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Reel {
    /// The recording.
    pub clip: Recorded,
    /// The speed picked; `Speed::PLAIN` plays it as recorded.
    pub speed: Speed,
}

impl Reel {
    /// Its length at the speed picked, in ms, rounded up.
    #[must_use]
    pub fn length_ms(self) -> u32 {
        self.clip.length_ms.div_ceil(u32::from(self.speed.times()))
    }
}

/// The picture editor, whose drawings its model, `edit::Canvas`, holds.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Edit {
    /// What was just done, which shows a short Copied or Saved.
    pub done: Option<Keep>,
}
