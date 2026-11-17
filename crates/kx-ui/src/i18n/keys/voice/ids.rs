//! Voice typing's ids: the marks beside rows, and each hint, button, page and place, which name their texts and which
//! QML sends back on a click. Each match makes a new one name its id.

use kx_module_api::ports::{BarButton, BarHint, BarPlace, FetchButton, PanelButton, VoiceTab};

/// A mark beside a row, by which QML also picks its icon.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VoiceMark {
    /// The model or device in use.
    Used,
    /// The language's default model.
    Default,
    /// A model whose file is here.
    Ready,
    /// A model to download.
    Download,
    /// A model too big for this PC.
    NotHere,
    /// A file no row names.
    Other,
    /// A card that passed its check.
    Passed,
    /// A card that failed its check.
    Failed,
    /// A model Compare ran.
    Done,
    /// The model Compare runs now.
    Writing,
    /// A model whose turn has not come.
    Waiting,
    /// A model Compare could not run, or skipped after its fault.
    RunFailed,
}

impl VoiceMark {
    /// Every mark.
    pub const ALL: [Self; 12] = [
        Self::Used,
        Self::Default,
        Self::Ready,
        Self::Download,
        Self::NotHere,
        Self::Other,
        Self::Passed,
        Self::Failed,
        Self::Done,
        Self::Writing,
        Self::Waiting,
        Self::RunFailed,
    ];
}

/// A mark's id.
#[must_use]
pub const fn mark_id(mark: VoiceMark) -> &'static str {
    match mark {
        VoiceMark::Used => "used",
        VoiceMark::Default => "default",
        VoiceMark::Ready => "ready",
        VoiceMark::Download => "download",
        VoiceMark::NotHere => "not-here",
        VoiceMark::Other => "other",
        VoiceMark::Passed => "passed",
        VoiceMark::Failed => "failed",
        VoiceMark::Done => "done",
        VoiceMark::Writing => "writing",
        VoiceMark::Waiting => "waiting",
        VoiceMark::RunFailed => "run-failed",
    }
}

/// A hint's id, which QML also picks the note's icon by.
#[must_use]
pub const fn bar_hint_id(hint: &BarHint) -> &'static str {
    match hint {
        BarHint::Nothing => "nothing",
        BarHint::Cancelled => "cancelled",
        BarHint::NoMic => "no-mic",
        BarHint::MicBusy => "mic-busy",
        BarHint::MicRefused => "mic-refused",
        BarHint::Unplugged => "unplugged",
        BarHint::NoModel { .. } => "no-model",
        BarHint::Language(_) => "language",
        BarHint::Checking => "checking",
        BarHint::NoLanguage(_) => "no-language",
        BarHint::NoFit => "no-fit",
        BarHint::Fault => "fault",
        BarHint::Pieces => "pieces",
        BarHint::TooLong => "too-long",
        BarHint::NoField => "no-field",
        BarHint::Stopped => "stopped",
        BarHint::CopyFailed => "copy-failed",
        BarHint::Password => "password",
        BarHint::Fell => "fell",
        BarHint::Print => "print",
        BarHint::Taken => "taken",
        BarHint::Full => "full",
        BarHint::Folder => "folder",
        BarHint::Net => "net",
        BarHint::Cloud { .. } => "cloud",
    }
}

/// A bar button's id, which QML sends back on a click; its word is `voice.` then the id.
#[must_use]
pub const fn bar_button_id(button: BarButton) -> &'static str {
    match button {
        BarButton::Cancel => "cancel",
        BarButton::Stop => "stop",
        BarButton::Copy => "copy",
        BarButton::Undo => "undo",
        BarButton::Anyway => "anyway",
        BarButton::Again => "again",
        BarButton::Listen => "listen",
        BarButton::Download => "download",
        BarButton::Close => "close",
    }
}

/// A panel button's id, which QML sends back on a click.
#[must_use]
pub const fn panel_button_id(button: PanelButton) -> &'static str {
    match button {
        PanelButton::Back => "back",
        PanelButton::Close => "close",
        PanelButton::Use => "use",
        PanelButton::Download => "download",
        PanelButton::Delete => "delete",
        PanelButton::Check => "check",
        PanelButton::Again => "again",
        PanelButton::Rerun => "rerun",
        PanelButton::Insert => "insert",
        PanelButton::Copy => "copy",
        PanelButton::Pin => "pin",
        PanelButton::DeleteAll => "delete-all",
        PanelButton::Cancel => "cancel",
        PanelButton::Change => "change",
        PanelButton::Open => "open",
        PanelButton::Here => "here",
    }
}

/// A download button's id, which QML sends back on a click; its word is `voice.` then the id.
#[must_use]
pub const fn fetch_button_id(button: FetchButton) -> &'static str {
    match button {
        FetchButton::Pause => "pause",
        FetchButton::Resume => "resume",
        FetchButton::Cancel => "cancel",
    }
}

/// A page's id, which QML sends back to open it; its name is `voice.` then the id.
#[must_use]
pub const fn voice_tab_id(tab: VoiceTab) -> &'static str {
    match tab {
        VoiceTab::Models => "models",
        VoiceTab::Cloud => "cloud",
        VoiceTab::Device => "device",
        VoiceTab::Compare => "compare",
        VoiceTab::History => "history",
        VoiceTab::Options => "options",
    }
}

/// True for a page the home list shows; the Cloud page joins once its page and texts are built (P11 part 2).
#[must_use]
pub const fn home_tab(tab: VoiceTab) -> bool {
    !matches!(tab, VoiceTab::Cloud)
}

/// A bar place's id, as the `place` setting names it.
#[must_use]
pub const fn place_id(place: BarPlace) -> &'static str {
    match place {
        BarPlace::Above => "above",
        BarPlace::Bottom => "bottom",
        BarPlace::Top => "top",
    }
}
