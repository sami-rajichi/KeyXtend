//! What reaches the actor's inbox: the bus's events, a recording's notes from its stream thread, and what each job off
//! its thread gives back. Pixels, paths and typed text ride only inside redacted types, and none of it prints.

use kx_module_api::ports::{
    AskError, ClipPart, FolderPath, FreezeError, Frozen, PanelKey, RecordError, RecordFacts,
    RecordNote, SnipAct, SnipFile, SnipNote,
};

use super::run::Run;
use crate::flow::{Call, Video};

/// A message for the actor, from the bus, a recording, a job or the module.
pub(crate) enum Msg {
    /// The Snip key was clicked (`SnipStart`).
    Key,
    /// A click on Snip's windows (`SnipAct`).
    Act(SnipAct),
    /// A key of the keyboard while a Snip window or the editor's text line is aimed at (`SnipLineKey`).
    LineKey(PanelKey),
    /// The keyboard asked for the key's light and the line (`SnipAsk`).
    Ask,
    /// A note of recording `video`, from its stream thread.
    Note {
        /// The recording.
        video: Video,
        /// The note.
        note: RecordNote,
    },
    /// A job's answer.
    Back(Back),
    /// The module stops.
    Quit,
}

/// How a Copy or Save job came out.
pub(crate) enum Answer {
    /// Copied, or written where the Save dialog said.
    Done,
    /// The Save dialog was cancelled.
    Cancelled,
    /// It failed; the note says why.
    Failed(SnipNote),
    /// Another app held the clipboard; the parts wait for another try.
    Busy(Vec<ClipPart>),
}

/// How the Copy or Save in flight came out.
pub(crate) struct Kept {
    /// The re-timed copy it was asked to make, if any.
    pub into: Option<Video>,
    /// The re-timed copy it wrote, if it made one and still holds it.
    pub made: Option<SnipFile>,
    /// The video it marked copied for the clipboard, which a stopped actor deletes unless a Copy of it was done.
    pub copied: Option<SnipFile>,
    /// How it came out.
    pub answer: Answer,
}

impl Kept {
    /// An answer with no video of its own.
    pub(crate) fn of(answer: Answer) -> Self {
        Self {
            into: None,
            made: None,
            copied: None,
            answer,
        }
    }
}

/// What a job off the actor's thread gives back.
pub(crate) enum Back {
    /// What the system can record now.
    Facts(RecordFacts),
    /// Freeze `n` froze the screens, or why not; only the newest freeze counts.
    Frozen {
        /// Which freeze, counted by the actor.
        n: u32,
        /// Its screens, or why not.
        answer: Result<Frozen, FreezeError>,
    },
    /// Recording `video` started, or why not.
    Started {
        /// The recording.
        video: Video,
        /// Its run, held by the actor until it ends; dropped unheard, it is cancelled and its video deleted.
        run: Result<Run, RecordError>,
    },
    /// A call on recording `video`'s run ended.
    Called {
        /// The recording.
        video: Video,
        /// The run, back for its next call.
        run: Run,
        /// The call made.
        call: Call,
        /// How it came out.
        done: Result<(), RecordError>,
    },
    /// The job in flight started writing its copy at a speed, after any Save dialog.
    Timing,
    /// The Copy or Save in flight came out.
    Kept(Kept),
    /// The folder dialog's answer, for the Save dialog's folder.
    Folder(Result<Option<FolderPath>, AskError>),
    /// A job that failed and has no other answer: the user's player or the file manager did not open.
    Note(SnipNote),
}
