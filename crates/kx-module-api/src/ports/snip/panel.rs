//! The Snip panel (design "The Snip panel"): the Capture page, with the keyboard during a capture and the folder the
//! Save dialog opens at, and the Video page's steppers and switches. A folder may name the user, so it never prints.

use crate::ports::{Ends, FolderPath};
use serde::{Deserialize, Serialize};

listed! {
    /// What the keyboard does during a capture, the `keyboard` setting.
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum SnipKeyboard ("Every choice, in the panel's order.") {
        /// It waits as its bubble and stays out of every capture.
        Bubble,
        /// Nothing of ours but the capture's own windows shows until it ends.
        Hidden,
        /// It stays in place and is in the picture or video.
        Shown,
    }
}

listed! {
    /// A page of the panel.
    pub enum SnipTab ("Both pages, in order.") {
        /// The keyboard during a capture, and the folder.
        Capture,
        /// The countdown, frames a second, the longest video and the sounds at start.
        Video,
    }
}

listed! {
    /// A row of the Video page.
    pub enum VideoRow ("Every row, in the page's order.") {
        /// The countdown, by − and +.
        Countdown,
        /// Frames a second, one of `RecordAsk::FPS`.
        Fps,
        /// The longest video, by − and +.
        Longest,
        /// The microphone at start: Off or On.
        Mic,
        /// The computer's sound at start: Off or On.
        Sound,
    }
}

listed! {
    /// A key of the folder row.
    pub enum FolderKey ("Both folder keys.") {
        /// Pick another folder in the system's folder dialog.
        Change,
        /// Show the folder in the file manager.
        Open,
    }
}

/// The Capture page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureSheet {
    /// The keyboard during a capture, with one line saying what it does.
    pub keyboard: SnipKeyboard,
    /// The folder the Save dialog opens at: `save_dir`, or the Snips folder; `None` while neither is there.
    pub folder: Option<FolderPath>,
}

/// The Video page.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VideoSheet {
    /// The countdown, in seconds (`countdown_s`).
    pub countdown_s: u8,
    /// Where it stands, so − or + dims at an end.
    pub countdown: Ends,
    /// Frames a second (`fps`).
    pub fps: u8,
    /// The longest video, in minutes (`max_min`).
    pub max_min: u16,
    /// Where it stands.
    pub longest: Ends,
    /// The microphone records from the start (`mic_on`).
    pub mic_on: bool,
    /// The computer's sound records from the start (`sound_on`).
    pub sound_on: bool,
}

/// The page the panel shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnipSheet {
    /// The Capture page.
    Capture(CaptureSheet),
    /// The Video page.
    Video(VideoSheet),
}
