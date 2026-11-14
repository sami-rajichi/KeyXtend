//! One dictation, from Start until its bar closes: its language and model, its run, the pieces written and what the
//! keyboard typed. Its text stays secret and is wiped when dropped.

use std::collections::BTreeMap;

use kx_module_api::ports::{LayoutChanged, SpeechAsk, VoiceLang, VoiceText};
use kx_module_api::text::SPACE;
use kx_module_api::{LINE_BREAKS, Redacted, Secret, Zeroizing, is_arabic, is_hidden, is_rtl};

use super::{Out, Ready, RunCall, State};

/// A keyboard language: its ISO 639 code and its name for the bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Lang {
    /// The code; empty when unknown.
    pub(super) iso: String,
    /// The name the bar and its notes show.
    pub(super) shown: VoiceLang,
}

impl Lang {
    /// No layout told yet.
    pub(super) fn unknown() -> Self {
        let shown = VoiceLang {
            name: String::new(),
            rtl: false,
            arabic: false,
        };
        Self {
            iso: String::new(),
            shown,
        }
    }
}

impl From<&LayoutChanged> for Lang {
    fn from(layout: &LayoutChanged) -> Self {
        let shown = VoiceLang {
            name: layout.name.clone(),
            rtl: layout.rtl,
            arabic: is_arabic(&layout.name),
        };
        Self {
            iso: layout.iso.clone(),
            shown,
        }
    }
}

/// What the keyboard typed of this dictation, which Undo insert erases while nothing else was typed.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) struct Typed {
    /// The characters typed, the suffix included.
    pub(super) count: u32,
}

/// A `TextAsk` the keyboard has not answered yet.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) struct Asking {
    /// Its round.
    pub(super) round: u32,
    /// The focused box when it was asked.
    pub(super) box_id: u64,
    /// It only erases, for Undo insert.
    pub(super) undo: bool,
}

/// One dictation.
#[derive(Debug)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each flag is one fact the run told about this dictation, set and read on its own."
)]
pub(super) struct Take {
    /// Names its run and its notes.
    pub(super) id: u32,
    /// The language, fixed at Start.
    pub(super) lang: Lang,
    /// The model and device that write it, Try again's choice once it ran.
    pub(super) ready: Ready,
    /// The ask the recording was last written with, which Transcribe anyway writes again.
    pub(super) ask: SpeechAsk,
    /// Its run is open: opened and not yet closed.
    pub(super) open: bool,
    /// The run still holds the recording.
    pub(super) held: bool,
    /// Speech was heard.
    pub(super) heard: bool,
    /// `record_max_s` stopped it.
    pub(super) long: bool,
    /// A card failed, so the processor wrote it.
    pub(super) fell: bool,
    /// The recording was asked for Keep recordings.
    pub(super) kept: bool,
    /// A text of it went to a password box, so each later one stays dots and is never kept.
    pub(super) masked: bool,
    /// The pieces written, by their place.
    pub(super) pieces: BTreeMap<u16, Secret>,
    /// The last text, without the suffix.
    pub(super) text: Option<Secret>,
    /// The Text bar Try again left, shown again when it writes nothing.
    pub(super) before: Option<State>,
    pub(super) typed: Option<Typed>,
    pub(super) asking: Option<Asking>,
}

impl Take {
    /// Dictation `id` in `lang`, opened with `ask` from `ready`.
    pub(super) fn new(id: u32, lang: Lang, ready: Ready, ask: SpeechAsk) -> Self {
        Self {
            id,
            lang,
            ready,
            ask,
            open: true,
            held: true,
            heard: false,
            long: false,
            fell: false,
            kept: false,
            masked: false,
            pieces: BTreeMap::new(),
            text: None,
            before: None,
            typed: None,
            asking: None,
        }
    }

    /// `call` on this take's run.
    pub(super) fn call(&self, call: RunCall) -> Out {
        Out::Run {
            take: self.id,
            call,
        }
    }

    /// The same recording is written again: its pieces and the card's fall start afresh.
    pub(super) fn redo(&mut self) {
        self.pieces.clear();
        self.fell = false;
    }

    /// The pieces in order on one line, one space apart, without hidden marks.
    pub(super) fn joined(&self) -> Zeroizing<String> {
        let room = self.pieces.values().map(|p| p.expose().len() + 1).sum();
        // Reserved up front, so no copy of the text is left behind by a growing buffer.
        let mut out = Zeroizing::new(String::with_capacity(room));
        let chars = (self.pieces.values()).flat_map(|p| p.expose().chars().chain([SPACE]));
        for c in chars.filter_map(typable) {
            if c != SPACE || !(out.is_empty() || out.ends_with(SPACE)) {
                out.push(c);
            }
        }
        while out.ends_with(SPACE) {
            out.pop();
        }
        out
    }
}

/// `c` as typed: a line break or another control reads as a space, another hidden mark goes, the rest stays.
fn typable(c: char) -> Option<char> {
    if c.is_control() || LINE_BREAKS.contains(&c) {
        Some(SPACE)
    } else if is_hidden(c) {
        None
    } else {
        Some(c)
    }
}

/// `text` with `suffix` after it, as the keyboard types it.
pub(super) fn suffixed(text: &str, suffix: &str) -> Secret {
    let mut out = Zeroizing::new(String::with_capacity(text.len() + suffix.len()));
    out.push_str(text);
    out.push_str(suffix);
    Redacted::new(out)
}

/// `text` as the bar shows it, with its way and font.
pub(super) fn shown(text: &str) -> VoiceText {
    VoiceText::Shown {
        text: Redacted::new(text.to_owned()),
        rtl: is_rtl(text),
        arabic: is_arabic(text),
    }
}

/// A copy of `text`, kept secret.
pub(super) fn secret(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}
