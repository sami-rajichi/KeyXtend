//! What the capture flow hears and asks for: the inputs in, the steps out, and the videos of the unsaved part it names.
//! The actor carries each step out on the ports and the bus, and answers with an input.

use kx_module_api::ports::{
    AskKey, BarKey, EditKey, FreezeError, Frozen, Handle, Keep, Part, Point, RecKey, RecordNote,
    Shot, SnipAside, SnipLit, SnipNote, Speed, StripKey,
};

use crate::SnipSettings;

/// A video of the Snips folder's unsaved part, by the flow's count: a recording, or a re-timed copy of one. The actor
/// maps it to its file.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Video(pub(super) u32);

/// How a Copy or Save the flow asked for, or the re-timing Play asked, came out.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Copied, written where the Save dialog said, or re-timed.
    Done,
    /// The Save dialog was cancelled.
    Cancelled,
    /// It failed; the note says why.
    Failed(SnipNote),
}

/// One input to the flow, from the bus, a port or the clock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum In {
    /// The Snip key was clicked (`SnipStart`).
    Key,
    /// Our keyboard's Esc, when the editor's text line did not take it.
    Esc,
    /// The ui's answer to `Out::Aside`: the keyboard stepped aside, so the screens may freeze.
    Aside,
    /// A capture window could not be kept out of captures: a countdown ends and a recording stops, as Stop does.
    Bare,
    /// The freeze's answer to `Out::Freeze`.
    Frozen(Result<Frozen, FreezeError>),
    /// A capture bar key.
    Bar(BarKey),
    /// A click on the frozen screen, in desktop pixels: the part rules read it for the way picked, the frame rules on a
    /// region's strip.
    Spot(Point),
    /// The theme's frame handle click area across, in logical pixels (`SnipFit::handle`).
    HitPx(u16),
    /// A reader's press on a region's frame handle, which picks it up at its spot.
    Handle(Handle),
    /// A strip key.
    Strip(StripKey),
    /// A record bar key.
    Record(RecKey),
    /// A note of recording `video`'s run, or its start's error told as a fault.
    Note {
        /// The recording, as `Out::Record` named it.
        video: Video,
        /// The note.
        note: RecordNote,
    },
    /// An editor key; Undo and Redo go to the editor's model, `edit::Canvas`, which tells `Changed`.
    Edit(EditKey),
    /// The editor's picture changed, as `edit::Did::Change` tells: a drawing, a crop, an erase, an undo or a redo.
    Changed,
    /// The editor's picture flattened, for its Copy or Save, or the question's Save. Before it, as before Close and the
    /// Snip key, the actor ends the text being typed (`edit::Canvas::end`) and tells `Changed` when that is a change.
    Flat {
        /// Copy or Save.
        keep: Keep,
        /// The pixels, the drawings drawn in.
        shot: Shot,
    },
    /// An answer to the close question.
    Ask(AskKey),
    /// How the Copy or Save, or the re-timing for Play, asked last came out.
    Kept(Outcome),
    /// The copy at a speed asked last started being written, after any Save dialog; its strip key waits until `Kept`.
    Timing,
    /// `[snip]` changed; a running snip keeps how it set the keyboard aside.
    Settings(Box<SnipSettings>),
    /// The time `Flow::deadline` named came.
    Tick,
    /// The module stops: a recording is cancelled and the snip ends.
    Quit,
}

/// A call on a recording's run (`RecordRun`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Call {
    /// Switch the microphone on or off.
    Mic(bool),
    /// Switch the computer's sound on or off.
    Sound(bool),
    /// Stop, keeping the video; `Done` follows.
    Stop,
    /// Stop at once and delete the video.
    Cancel,
}

/// A re-timed copy to write first, into a new video of the unsaved part.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timed {
    /// How many times faster it plays.
    pub speed: Speed,
    /// The new video.
    pub into: Video,
}

/// What a Copy or Save keeps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum What {
    /// A picture.
    Picture(Shot),
    /// A recording, as recorded or re-timed first.
    Video {
        /// The recording.
        video: Video,
        /// The re-timed copy kept instead; `None` keeps it as recorded.
        timed: Option<Timed>,
    },
}

/// One step the actor carries out, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Out {
    /// The Snip key's light (`SnipLit`).
    Lit(SnipLit),
    /// The keyboard steps aside or comes back (`SnipAside`); a step aside waits for `In::Aside` before the freeze.
    Aside(SnipAside),
    /// Freeze every screen; `In::Frozen` answers.
    Freeze,
    /// Show a note for a moment.
    Note(SnipNote),
    /// Open the Snip panel, from the capture bar's settings key.
    Panel,
    /// Record `part` into new video `video` with its sound switches; its notes come as `In::Note`.
    Record {
        /// The new video, which its notes name.
        video: Video,
        /// The part filmed.
        part: Part,
        /// The microphone records from the start.
        mic: bool,
        /// The computer's sound records from the start.
        sound: bool,
    },
    /// Call recording `video`'s run.
    Run {
        /// The recording.
        video: Video,
        /// The call.
        call: Call,
    },
    /// Open the editor on this picture; the keyboard comes back.
    Edit(Shot),
    /// Copy, or ask the Save dialog and write there; `In::Kept` answers.
    Keep {
        /// Copy or Save.
        keep: Keep,
        /// What is kept.
        what: What,
    },
    /// Play this video in the user's own player.
    Play(Video),
    /// Re-time recording `video` into its copy for Play; `In::Kept` answers, and the copy plays once made.
    Retime {
        /// The recording.
        video: Video,
        /// The copy to write.
        timed: Timed,
    },
    /// Delete this video of the unsaved part.
    Delete(Video),
    /// The snip ended: its windows close, and the editor and the frozen pixels go.
    End,
}
