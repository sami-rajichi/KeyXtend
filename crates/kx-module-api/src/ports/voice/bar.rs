//! What the dictation bar shows (design "The bar"): its place, its state, its hint, its text and the cloud mark. Text
//! stays redacted and is wiped on drop, and a password box's text never reaches the ui.

use super::act::BarButton;
use super::panel::{Fact, ServiceName, Unavailable, VoicePanel};
use crate::Secret;
use crate::ports::LocalTime;
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
        /// The text, wiped when the last view holding it drops.
        text: Secret,
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
    /// The service's card warning, a reminder while the first dictation to it at its terms number listens.
    pub warn: Option<Fact>,
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
    /// Typing stopped part way, as the user's own key or click came: the whole text stays on the bar with Copy.
    Stopped,
    /// Copy failed, as another app held the clipboard: the text stays on the bar, so Copy may be tried again.
    CopyFailed,
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
    /// A cloud dictation's voice brought no text back, or never left (ADR-0047): the recording stays for Try again and
    /// "Use the local model".
    Cloud {
        /// Why.
        why: CloudWhy,
        /// "Use the local model".
        local: LocalUse,
    },
}

/// When a used free allowance renews: its local wall time, and whether that falls on the day the note came.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Renewal {
    /// The local wall time, in the offset in force then, so a summer-time change between shows its own hour.
    pub at: LocalTime,
    /// It falls on the day the note came, so the clock time alone names it.
    pub today: bool,
}

/// Why a cloud dictation's voice brought no text back, or never left; each names one line of the bar's cloud note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CloudWhy {
    /// The network failed, the service is down, or it gave no answer within `net_wait_s`.
    Net,
    /// The service refused the key, or none could be read, as one removed or damaged.
    Key,
    /// The account has no credit left.
    Credit,
    /// The service asks for a step on its site first, such as a verification, a payment method or a spending cap.
    Account,
    /// The free allowance is used up.
    Allowance {
        /// When it renews, from when the fault came, when the service tells.
        renews_at: Option<Renewal>,
    },
    /// Too many requests, still after one wait.
    Rate,
    /// A piece was too big for the service.
    TooBig,
    /// The service's model does not take this language's code, such as a regional one.
    Language(VoiceLang),
    /// The service is not offered in the user's country or region.
    Region,
    /// The service sent no text.
    Empty,
    /// The reply was not the service's, such as a hotspot's sign-in page.
    Strange,
    /// The service's terms changed since they were confirmed, so nothing is sent until they are confirmed again.
    Terms,
    /// The service was switched off, so nothing was sent.
    Off,
    /// The language's pick moved to another service or to a local model, so nothing was sent.
    Moved,
    /// A password box never goes to a cloud service, and no local model is ready for it.
    NoLocal,
    /// The cloud worker failed.
    Fault,
    /// The pick's model cannot be used now, for this reason, so nothing is sent until the user changes it.
    Unavailable(Unavailable),
}

impl CloudWhy {
    /// True for a note the service's own card or key page settles, so its bar offers Open service (owner, 2026-11-20).
    #[must_use]
    pub const fn opens(&self) -> bool {
        matches!(
            self,
            Self::Key | Self::Terms | Self::Account | Self::Allowance { .. } | Self::Off
        )
    }
}

/// "Use the local model" on a cloud note, which writes the held recording with the language's local model.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LocalUse {
    /// A local model is ready, so the button acts.
    Ready,
    /// None is ready: the button stays dimmed, and Download fetches the language's model.
    Download {
        /// The download's size, in bytes.
        bytes: u64,
    },
    /// None is ready, or the recording is gone, so the button stays dimmed.
    Dimmed,
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
    /// The service a cloud dictation sends the voice to at Stop; while set, the bar shows the cloud mark and its name.
    pub cloud: Option<ServiceName>,
}
