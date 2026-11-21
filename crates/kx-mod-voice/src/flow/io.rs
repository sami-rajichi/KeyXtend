//! What the dictation flow hears and asks for: the inputs in, the effects out, and the model and device a language
//! gets, which the actor answers from the models table, the device rule and the folder.

use kx_module_api::ports::{
    BarButton, CardId, CardUse, FieldChange, Fingerprint, LayoutChanged, Renewal, SessionNote,
    SpeechAsk, SpeechDone, SpeechModel, SpeechNote, TextAsk, TextDone, VoiceAct, VoiceLevel,
    VoiceLit, VoicePanel, VoiceView,
};
use kx_module_api::{Redacted, Secret};

use super::{Lend, Net, Sent, Upload};
use crate::VoiceSettings;
use crate::device::Why;

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
    Panel(Option<Box<VoicePanel>>),
    /// The gear or the tray opens the panel (`VoiceOpen`); a running dictation or Compare's recording ends first.
    Opening,
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
    /// The clipboard took the bar's text (Copy), or refused it.
    Copied(bool),
    /// `[voice]` changed.
    Settings(Box<VoiceSettings>),
    /// The time `Flow::deadline` named came: a hint or a fade.
    Tick,
    /// Compare models holds the microphone's run, or lets it go (`Compare::lend`).
    Lend(Lend),
    /// Dictation `take`'s upload of round `round` tells `sent`; an older round's are dropped.
    Sent {
        /// The dictation, as `Out::Send` named it.
        take: u32,
        /// The upload's round.
        round: u32,
        /// What it tells.
        sent: Sent,
        /// When a used allowance renews, which the actor reads on the calendar as the fault comes.
        renews: Option<Renewal>,
    },
    /// A service's key was set or removed, so a running cloud dictation checks its own.
    Keys,
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
    /// The model that wrote it, as `Ready::name` names it, or the cloud pick, `service:model`.
    pub model: Redacted<String>,
}

/// A written dictation's numbers for the counts; never its content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Count {
    /// The model that wrote it, or the cloud pick, `service:model`.
    pub model: Redacted<String>,
    /// The card that wrote it to the end; `None` for the processor or a cloud service.
    pub card: Option<CardId>,
    /// Its times and memory.
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
    /// The key's light: off, amber or red (`VoiceLit`).
    Lit(VoiceLit),
    /// Tell the sound level while listening (`VoiceLevel`).
    Level(VoiceLevel),
    /// Ask the keyboard to erase, then type (`TextAsk`).
    Type(TextAsk),
    /// Erase as `Type` does, then put the text in at once through the clipboard history; typed should that fail.
    Paste(TextAsk),
    /// Put this text on the clipboard (Copy).
    Copy(Secret),
    /// Keep this text in History, whose `Rules` drop it while `history` is off.
    Keep(Kept),
    /// Count a written dictation.
    Count(Count),
    /// Download the model of this keyboard language, an ISO 639 code.
    Download(String),
    /// The model's file was not found: read the folder again, so its row offers Download.
    Lost,
    /// The card failed mid-run: drop its check result for this model file, so that check runs again.
    Fell {
        /// The card.
        card: CardId,
        /// The model file's fingerprint.
        model: Fingerprint,
    },
    /// Compare holds the run: give it this Stop or Cancel (`Panel::click` with `VoiceClick::Bar`).
    Lent(BarButton),
    /// Send the dictation's speech, which its run hands over, to the service the upload names.
    Send(Upload),
    /// Drop dictation `take`'s running upload: nothing more is sent or heard of it.
    Halt {
        /// The dictation.
        take: u32,
    },
    /// The first dictation to this service at its row's terms number showed the card's warning: note it as seen.
    Warned(String),
    /// Open service on a cloud note: the panel opens on this service's card, or on its key page when `key`.
    Settle {
        /// The service's row id.
        service: String,
        /// The key note, so its key page opens.
        key: bool,
    },
}

/// The model and device that serve a language now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ready {
    /// The model a job loads, with the model's own code for the language.
    pub model: SpeechModel,
    /// The card that runs it; `None` runs it on the processor.
    pub card: Option<CardUse>,
    /// Why the processor runs it although a card is picked; `None` otherwise.
    pub why: Option<Why>,
    /// Its row's id or its file's name, for History and the counts.
    pub name: Redacted<String>,
    /// Its file's fingerprint, which a card's check goes by.
    pub print: Fingerprint,
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
    /// The picked model does not know the language, or its file cannot be read.
    Stranger,
    /// The picked file is still being read for its fingerprint or its languages.
    Checking,
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

    /// The cloud service the user picked for keyboard language `iso`, while cloud services and that service are on.
    fn cloud(&self, _iso: &str) -> Option<Net> {
        None
    }

    /// True while `service` is switched on with cloud services on, so a pick that left it moved rather than went off.
    fn on(&self, _service: &str) -> bool {
        false
    }
}

impl<F: Fn(&str) -> Result<Ready, Lack>> Supply for F {
    fn supply(&self, iso: &str) -> Result<Ready, Lack> {
        self(iso)
    }
}
