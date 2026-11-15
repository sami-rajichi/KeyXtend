//! What the dictation bar shows (design "The bar"): its place, its state, its hint and its text. Text stays redacted,
//! and a password box's text never reaches the ui.

use super::act::BarButton;
use super::panel::VoicePanel;
use crate::Redacted;
use serde::{Deserialize, Serialize};

listed! {
    /// Where the bar goes, from the `place` setting.
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum BarPlace ("Every place.") {
        /// Just above the keyboard and centred on it, never over it; the bottom centre of its screen with no room.
        Above,
        /// The bottom centre of the screen.
        Bottom,
        /// The top centre of the screen.
        Top,
    }
}

/// A language as the bar's chip, a hint and the panel's header name it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoiceLang {
    /// Its name for the user, as `LayoutChanged::name` gives it.
    pub name: String,
    /// Its ISO 639 code, such as `fr`, as `LayoutChanged::iso` gives it; empty when unknown.
    pub iso: String,
    /// It is written right to left.
    pub rtl: bool,
    /// It needs the Arabic font.
    pub arabic: bool,
}

/// The text the bar shows; Copy shows only for a shown text, so a password box never offers it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VoiceText {
    /// The text typed or kept; the bar shows its first lines, with Copy.
    Shown {
        /// The text.
        text: Redacted<String>,
        /// It reads right to left.
        rtl: bool,
        /// It needs the Arabic font.
        arabic: bool,
    },
    /// A password box took it, so the bar shows dots, carries no text and offers no Copy.
    Masked,
}

/// The Listening state: the red dot, the language, the live wave, the timer and the gear.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Listening {
    /// The language, fixed at Start.
    pub lang: VoiceLang,
    /// How long it has listened when this view was made, in ms; the ui's timer counts on from it.
    pub elapsed_ms: u32,
    /// No speech came for `nothing_s`, so the bar hints at it and keeps listening.
    pub nothing: bool,
    /// How long the wave smooths the level over, in ms (`wave_ms`).
    pub wave_ms: u16,
}

/// The Text state: the text, a hint under it, and Undo insert, which stays in its place dimmed once it cannot act.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BarText {
    /// The text.
    pub text: VoiceText,
    /// A small line under it, such as Too long or Password, if any.
    pub hint: Option<BarHint>,
    /// Undo insert acts: nothing else was typed since.
    pub undo: bool,
}

/// The Note state: one line saying what happened, with the action it names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BarNote {
    /// What happened.
    pub hint: BarHint,
    /// The action it names, such as Listen again, Transcribe anyway or Download, if any.
    pub act: Option<BarButton>,
    /// The microphone is still open, so the red dot stays.
    pub mic: bool,
}

/// The bar's state; the gear shows in each one but `Hidden`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VoiceBar {
    /// No bar.
    Hidden,
    /// The microphone listens.
    Listening(Listening),
    /// The last piece is being written: Cancel acts, and Stop stays in its place dimmed.
    Transcribing,
    /// The text was typed, or kept on the bar when nothing took it.
    Text(BarText),
    /// A note.
    Note(BarNote),
}

/// A line the bar or the panel's band shows; each names one text in every language, with the facts it prints.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BarHint {
    /// Stop came with no speech, so nothing was typed: Listen again acts.
    Nothing,
    /// Cancel typed nothing; Transcribe anyway acts until the bar closes.
    Cancelled,
    /// No microphone was found.
    NoMic,
    /// Another app holds the microphone.
    MicBusy,
    /// The system's privacy switch refused the microphone.
    MicRefused,
    /// The microphone was unplugged while listening.
    Unplugged,
    /// The language has no model in the folder, or its file vanished: Download acts, with the default's size.
    NoModel {
        /// The language, as in No model for English yet.
        lang: VoiceLang,
        /// The size of the model Download fetches, in bytes, as in Download · 740 MB.
        bytes: u64,
    },
    /// The picked model does not know the language: the gear acts.
    Language(VoiceLang),
    /// The picked file is still being read for its fingerprint or its languages.
    Checking,
    /// No listed model knows the language.
    NoLanguage(VoiceLang),
    /// The model does not fit in the memory free now.
    NoFit,
    /// The voice worker failed.
    Fault,
    /// The worker ended mid-way: the pieces already written show, with Copy.
    Pieces,
    /// `record_max_s` stopped the dictation, then its text was typed.
    TooLong,
    /// Nothing in front took text, so it stays on the bar with Copy and in History.
    NoField,
    /// A password box took the text: it shows as dots and is never kept.
    Password,
    /// The card failed, so the processor wrote the text.
    Fell,
    /// A download had the wrong size or fingerprint, so its part was deleted.
    Print,
    /// Another file in the models folder holds the model's name, so nothing was downloaded.
    Taken,
    /// The drive is full, or under `min_free_mb`, so nothing was downloaded.
    Full,
    /// The models folder is gone, read-only or refused.
    Folder,
    /// A download failed, stalled or met a host its row does not list.
    Net,
}

/// Everything the bar and the panel draw.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoicePage {
    /// Counts each new view, so a click names the view it saw.
    pub round: u32,
    /// The bar.
    pub bar: VoiceBar,
    /// The panel, while it is open, above the bar.
    pub panel: Option<VoicePanel>,
    /// Where the bar goes.
    pub place: BarPlace,
    /// The start and stop sounds play (`sounds`).
    pub sounds: bool,
}
