//! What Snip's windows draw (design "What the user sees"): the frozen screens, the capture bar, the strip, the
//! countdown, the record bar, the editor, the close question, a note and the panel, each key with why it dims.

use super::editor::Editor;
use super::frame::Frame;
use super::panel::SnipSheet;
use super::shot::{FrozenScreen, Part};
use super::speed::Speed;
use crate::ports::{Point, Rect};

listed! {
    /// A key of the capture bar.
    pub enum BarKey ("Every bar key, in the spec's order.") {
        /// Click a corner, move, click the opposite corner.
        Region,
        /// Click a window.
        Window,
        /// Click a screen.
        Full,
        /// Record a region.
        VideoRegion,
        /// Record a screen.
        VideoFull,
        /// Open the Snip panel.
        Settings,
        /// End the capture, keeping nothing.
        Close,
    }
}

listed! {
    /// A key of the strip beside a part.
    pub enum StripKey ("Every strip key.") {
        /// Put the snip on the clipboard.
        Copy,
        /// Open the editor on a picture.
        Edit,
        /// Show the system's Save dialog.
        Save,
        /// End the snip, asking first when it was not kept.
        Close,
        /// Count down, then record.
        Record,
        /// The microphone's switch.
        Mic,
        /// The computer sound's switch.
        Sound,
        /// Open the recorded video at its speed in the user's own player, its re-timed copy written first.
        Play,
        /// The recorded video's next speed of `SpeedAsk::SPEEDS`, back to the first after the last.
        Speed,
    }
}

listed! {
    /// A key of the record bar.
    pub enum RecKey ("Every record bar key.") {
        /// The microphone's switch.
        Mic,
        /// The computer sound's switch.
        Sound,
        /// End the recording, keeping it.
        Stop,
        /// End the recording and delete it.
        Cancel,
    }
}

listed! {
    /// An answer to the close question.
    pub enum AskKey ("Every answer, in the question's order.") {
        /// Save the snip first.
        Save,
        /// Delete it for good.
        Drop,
        /// Back to the strip or the editor, as Esc does.
        Back,
    }
}

listed! {
    /// How a snip was kept, which shows a tick and Copied or Saved.
    pub enum Keep ("Both ways.") {
        /// Put on the clipboard.
        Copy,
        /// Saved where the Save dialog said.
        Save,
    }
}

listed! {
    /// Where the strip or the record bar goes.
    pub enum SnipSpot ("Every place.") {
        /// Under the part.
        Under,
        /// Above the part.
        Above,
        /// At the bottom centre of its screen.
        Bottom,
        /// At the top centre of its screen.
        Top,
    }
}

listed! {
    /// Why a key stays dimmed; each names its tip.
    pub enum Dim ("Every reason the spec names.") {
        /// A hidden run (ADR-0037) made no Snips folder: Video and Save dim.
        Hidden,
        /// A capture window could not be kept out of captures, so a hidden record bar could not stop a video.
        Bare,
        /// The system has no video encoders, as a Windows N edition without its Media Feature Pack.
        NoVideo,
        /// No microphone was found.
        NoMic,
        /// The system's privacy switch refuses the microphone.
        MicRefused,
        /// No sound device was found.
        NoSound,
        /// The drive holds less than `min_free_mb`.
        DiskLow,
    }
}

listed! {
    /// A note a capture window shows for a moment.
    pub enum SnipNote ("Every note.") {
        /// The screens together pass `shot_cap_mb`, so nothing was frozen.
        TooBig,
        /// The system refused to grab the screens, so nothing was frozen.
        FreezeFailed,
        /// The drive is under `min_free_mb`: Record dims, or a recording stopped and kept its video.
        DiskLow,
        /// `max_min` stopped the recording, which kept its video.
        Longest,
        /// The system has no video encoders.
        NoVideo,
        /// The recording failed, and its video was deleted.
        RecordFailed,
        /// The snip could not be saved there.
        SaveFailed,
        /// Another app held the clipboard, so nothing was copied.
        ClipBusy,
        /// The clipboard refused the snip for another reason, so nothing was copied.
        CopyFailed,
        /// The video could not be made at its new speed; the recording stays as it was.
        SpeedFailed,
        /// A capture window could not be kept out of captures, so the countdown ended or the recording stopped.
        Bare,
        /// The Save or folder dialog's answer lies off a drive or in `unsaved/`, so the user must choose another.
        PlaceRefused,
        /// The user's player did not open the video.
        PlayFailed,
        /// The file manager or the folder dialog did not open.
        FolderFailed,
        /// The microphone could not record, at the start or when switched on, so Mic turned off.
        MicFailed,
        /// The computer's sound could not record, at the start or when switched on, so Sound turned off.
        SoundFailed,
    }
}

/// A key a capture window shows: what it is, whether it is on, and why it stays dimmed, if it does.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Keycap<K> {
    /// The key.
    pub key: K,
    /// The way picked, or a sound switch that records.
    pub on: bool,
    /// Why it stays dimmed; `None` while it acts.
    pub dim: Option<Dim>,
}

/// Picking: the capture bar over the frozen screens, a way picked, and the first corner while a region waits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Picking {
    /// The screen whose top centre holds the bar: the one the pointer was on.
    pub screen: Rect,
    /// The bar's keys in order; the hint under it follows the way picked.
    pub keys: Vec<Keycap<BarKey>>,
    /// The first corner clicked, in desktop pixels, while the second is awaited.
    pub corner: Option<Point>,
    /// The screen a video region keeps to once its first corner is clicked, so the shown rectangle stops at its edge.
    pub within: Option<Rect>,
    /// What Window offers, front first, so the ui outlines the one under the pointer; empty for another way.
    pub windows: Vec<Rect>,
}

/// A part taken: it stays clear, with the strip beside it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Taken {
    /// The part, clear on the frozen screens, or outlined on the live screen once its video recorded.
    pub part: Part,
    /// The screen holding it, whose bottom centre takes the strip for a full screen; its work area once the video
    /// recorded, when one was told (`SnipClick::Screens`).
    pub screen: Rect,
    /// Where the strip goes.
    pub spot: SnipSpot,
    /// The strip's keys in order; none while a frame's handle is held, when the strip hides.
    pub keys: Vec<Keycap<StripKey>>,
    /// The recorded video's length in ms at its speed, shown first; `None` for a picture or before recording.
    pub length_ms: Option<u32>,
    /// The video's speed, which Speed shows and Play, Copy and Save use; `None` for a picture or before recording.
    pub speed: Option<Speed>,
    /// What was just done, shown with a tick.
    pub done: Option<Keep>,
    /// The key that waits while the video's copy at its speed is written, after any Save dialog: Play, Copy or Save.
    pub waits: Option<StripKey>,
    /// A region's frame on the frozen screens; `None` for a window, a screen or a recorded video.
    pub frame: Option<Frame>,
}

/// The countdown before a video, on the live screen.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Countdown {
    /// The part, outlined.
    pub part: Part,
    /// The seconds left, shown in its centre.
    pub left_s: u8,
}

/// Recording: the red edge around the part, and the record bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recording {
    /// The part filmed.
    pub part: Part,
    /// The screen holding it, whose top centre takes the bar for a full screen; its work area when one was told.
    pub screen: Rect,
    /// Where the record bar goes.
    pub spot: SnipSpot,
    /// How long it has recorded when this view was made, in ms; the ui's timer counts on from it.
    pub elapsed_ms: u32,
    /// The run recorded its first frame; until then the ui's timer holds at nought.
    pub live: bool,
    /// The bar's keys in order.
    pub keys: Vec<Keycap<RecKey>>,
}

/// The capture's windows now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnipStage {
    /// None shows.
    Idle,
    /// The capture bar over the frozen screens.
    Pick(Picking),
    /// A part taken, with the strip.
    Strip(Taken),
    /// The countdown before a video.
    Count(Countdown),
    /// Recording.
    Record(Recording),
    /// The editor, boxed as it holds the most.
    Edit(Box<Editor>),
}

/// The close question: "This snip is not saved. Save it before closing?"
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CloseAsk {
    /// Why Save is dimmed, if it is.
    pub save_dim: Option<Dim>,
}

/// Everything Snip's windows draw, told once per change and shared.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnipPage {
    /// Counts each new view, so a click names the view it saw.
    pub round: u32,
    /// The frozen screens the overlay draws, dimmed but for the part; empty while the screen is live.
    pub frozen: Vec<FrozenScreen>,
    /// The capture's windows.
    pub stage: SnipStage,
    /// The close question, over the strip or the editor, while it shows.
    pub ask: Option<CloseAsk>,
    /// A note, if any.
    pub note: Option<SnipNote>,
    /// The Snip panel, above the keyboard, while it is open.
    pub panel: Option<SnipSheet>,
    /// The system's Save or folder dialog shows until it answers, so Snip's windows leave the top for it.
    pub dialog: bool,
}
