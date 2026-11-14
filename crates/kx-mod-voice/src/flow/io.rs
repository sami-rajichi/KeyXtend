//! What the dictation flow hears and asks for: the inputs in, the effects out, and the model and device a language
//! gets, which the actor answers from the models table, the device rule and the folder.

use kx_module_api::ports::{
    CardId, CardUse, FieldChange, LayoutChanged, SessionNote, SpeechAsk, SpeechDone, SpeechModel,
    SpeechNote, TextAsk, TextDone, VoiceAct, VoiceLevel, VoiceLit, VoicePanel, VoiceView,
};
use kx_module_api::{Redacted, Secret};

use crate::VoiceSettings;

/// One input to the flow, from the bus, a port or the clock.
#[derive(Clone, Debug)]
pub enum In {
    /// The Dictation key was clicked (`VoiceKey`).
    Key,
    /// A click on the bar or the panel (`VoiceAct`); the flow takes the bar's buttons, the pointer and Try again.
    Act(VoiceAct),
    /// A late starter asked for the light and the view again (`VoiceAsk`).
    Ask,
    /// The panel to draw above the bar, or `None` once it closed; an open panel keeps the bar.
    Panel(Option<VoicePanel>),
    /// A note from dictation `take`'s run, or a call's error told as a fault; an older take's notes are dropped.
    Note {
        /// The dictation the run belongs to, as `Out::Open` named it.
        take: u32,
        /// The note.
        note: SpeechNote,
    },
    /// The focused box changed (`FieldWatch`).
    Field(FieldChange),
    /// A key was typed or a click landed outside our windows, so Undo insert can no longer act.
    Elsewhere,
    /// The session locked, unlocked, slept or woke (`SessionWatch`).
    Session(SessionNote),
    /// The keyboard shows another layout (`LayoutChanged`).
    Layout(LayoutChanged),
    /// The keyboard typed a `TextAsk` (`TextDone`).
    Typed(TextDone),
    /// `[voice]` changed.
    Settings(Box<VoiceSettings>),
    /// The time `Flow::deadline` named came: a hint or a fade.
    Tick,
    /// The module stops: the microphone closes and the sound is wiped.
    Quit,
}

/// A call on the open dictation's run (`SpeechRun`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunCall {
    /// Stop listening and write the last piece.
    Stop,
    /// Stop listening or writing at once; the recording stays.
    Cancel,
    /// Write the held recording again as this ask says.
    Again(SpeechAsk),
    /// Hand over the recording for Keep recordings.
    Keep,
    /// Wipe the recording and end the run.
    Close,
}

/// A text for History; a later one of the same dictation, from Try again, replaces it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Kept {
    /// The dictation.
    pub take: u32,
    /// The text, without the suffix typed after it.
    pub text: Secret,
    /// The language it was dictated in, an ISO 639 code.
    pub lang: String,
    /// The model that wrote it, as `Ready::name` names it.
    pub model: Redacted<String>,
}

/// A written dictation's numbers for the counts; never its content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Count {
    /// The model that wrote it.
    pub model: Redacted<String>,
    /// Its times, memory and device.
    pub done: SpeechDone,
}

/// One effect the actor carries out, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Out {
    /// Open dictation `take`'s run as `ask` says; the microphone opens at once.
    Open {
        /// The dictation, which its notes name.
        take: u32,
        /// The ask.
        ask: SpeechAsk,
    },
    /// Call dictation `take`'s run.
    Run {
        /// The dictation.
        take: u32,
        /// The call.
        call: RunCall,
    },
    /// Tell the bar and the panel (`VoiceView`).
    View(VoiceView),
    /// Light the key, or not (`VoiceLit`).
    Lit(VoiceLit),
    /// Tell the sound level while listening (`VoiceLevel`).
    Level(VoiceLevel),
    /// Ask the keyboard to erase, then type (`TextAsk`).
    Type(TextAsk),
    /// Put this text on the clipboard (Copy).
    Copy(Secret),
    /// Keep this text in History; the store drops it while `history` is off.
    Keep(Kept),
    /// Count a written dictation.
    Count(Count),
    /// Download the model of this keyboard language, an ISO 639 code.
    Download(String),
    /// The model's file was not found: read the folder again, so its row offers Download.
    Lost,
    /// The card failed mid-run: drop its check result, so the check runs again.
    Fell(CardId),
}

/// The model and device that serve a language now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ready {
    /// The model a job loads, with the model's own code for the language.
    pub model: SpeechModel,
    /// The card that runs it; `None` runs it on the processor.
    pub card: Option<CardUse>,
    /// Its row's id or its file's name, for History and the counts.
    pub name: Redacted<String>,
    /// The size Download fetches for the language, in bytes, should the file be gone.
    pub bytes: u64,
}

/// Why no model serves a language now.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Lack {
    /// The language's model is not in the folder: Download fetches this many bytes.
    Missing {
        /// The download's size.
        bytes: u64,
    },
    /// The picked model does not know the language.
    Stranger,
    /// No listed model knows the language.
    Unknown,
    /// The model does not fit in the memory free now.
    NoFit,
}

/// Answers which model and device serve a keyboard language, from the models table, `picks`, `device` and the folder.
pub trait Supply {
    /// What serves keyboard language `iso` now, or why nothing does.
    ///
    /// # Errors
    /// The `Lack` the bar's note names.
    fn supply(&self, iso: &str) -> Result<Ready, Lack>;
}

impl<F: Fn(&str) -> Result<Ready, Lack>> Supply for F {
    fn supply(&self, iso: &str) -> Result<Ready, Lack> {
        self(iso)
    }
}
