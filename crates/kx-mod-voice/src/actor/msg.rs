//! What reaches the actor's inbox: the bus's and the watches' inputs, and what each job off its thread gives back. A
//! text, a recording or a path rides only inside redacted types or the runs, and none of it prints.

use std::path::PathBuf;

use kx_module_api::ports::{
    CardId, CheckDone, ClipTaken, CloudFault, CloudNote, Device, FetchNote, FieldChange,
    Fingerprint, FolderError, LayoutChanged, ModelError, ModelFacts, SessionNote, SpeechError,
    SpeechNote, TextDone, TextPasted, VoiceAct,
};

use super::ask::Wired;
use super::lanes::{Answer, Lane, Run};
use crate::panel::Check;

/// A message for the actor, from the bus, a watch, a job or the module.
pub(crate) enum Msg {
    /// The Dictation key was clicked (`VoiceKey`).
    Key,
    /// A click on the bar or the panel (`VoiceAct`).
    Act(VoiceAct),
    /// The tray or the gear opens the panel (`VoiceOpen`).
    Open,
    /// A late starter asks for the view and the light again (`VoiceAsk`).
    Ask,
    /// The keyboard shows another layout.
    Layout(LayoutChanged),
    /// The focused box changed.
    Field(FieldChange),
    /// The session locked, unlocked, slept or woke.
    Session(SessionNote),
    /// A key was typed or a click landed outside our windows.
    Elsewhere,
    /// The mouse module asks whether presses are wanted.
    PressesAsk,
    /// The keyboard typed a text (`TextDone`).
    Typed(TextDone),
    /// The clipboard history pasted a text, or not (`TextPasted`).
    Pasted(TextPasted),
    /// The clipboard history's text for the key page's Paste (`ClipTaken`).
    Taken(ClipTaken),
    /// A job's answer.
    Back(Back),
    /// The module stops.
    Quit,
}

/// What a job off the actor's thread gives back.
pub(crate) enum Back {
    /// A run's note.
    Note {
        /// The run.
        lane: Lane,
        /// The note.
        note: SpeechNote,
    },
    /// A run opened, or why not.
    Opened {
        /// The run.
        lane: Lane,
        /// The run itself, back for its calls.
        run: Result<Run, SpeechError>,
    },
    /// A run's call ended.
    Called {
        /// The run.
        lane: Lane,
        /// The run itself, back for its next call; `None` once closed.
        run: Option<Run>,
        /// What the call gave.
        answer: Result<Answer, SpeechError>,
    },
    /// The devices as read.
    Devices(Result<Vec<Device>, SpeechError>),
    /// An other file's facts, by its fingerprint.
    Facts(Fingerprint, Result<ModelFacts, SpeechError>),
    /// A file's fingerprint as read.
    Printed {
        /// The file.
        file: PathBuf,
        /// Its size as scanned.
        bytes: u64,
        /// Its fingerprint.
        print: Result<Fingerprint, ModelError>,
    },
    /// A card's warm-up moved on.
    Warming {
        /// The card.
        card: CardId,
        /// Its percent.
        percent: u8,
    },
    /// A card's check ended.
    Checked(Box<Check>, Result<CheckDone, SpeechError>),
    /// A download's note.
    Fetched {
        /// The row's id.
        row: String,
        /// The note.
        note: FetchNote,
    },
    /// The models folder was shown, or why not.
    Shown(Result<(), FolderError>),
    /// A note from dictation `take`'s upload of round `round`.
    Net {
        /// The dictation.
        take: u32,
        /// The upload.
        round: u32,
        /// The note; a text stays secret.
        note: CloudNote,
    },
    /// A key's test of `ticket` answered.
    Tested {
        /// Names the test, so an answer the user no longer waits for is dropped.
        ticket: u32,
        /// Works, or why not.
        answer: Result<(), CloudFault>,
    },
    /// A note from cloud model `at` of Compare's run `take`.
    Trial {
        /// Compare's run.
        take: u32,
        /// The model, in the run's order.
        at: usize,
        /// The note; a text stays secret.
        note: CloudNote,
    },
    /// Model `at`'s service run for Compare's run `take` opened.
    Tried {
        /// Compare's run.
        take: u32,
        /// The model.
        at: usize,
        /// The run, held until it ends; dropping it cancels its requests.
        run: Wired,
    },
    /// The service's run for dictation `take`'s upload of round `round` opened.
    Sending {
        /// The dictation.
        take: u32,
        /// The upload.
        round: u32,
        /// The run, held until it ends; dropping it cancels its requests.
        run: Wired,
    },
}
