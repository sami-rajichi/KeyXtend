//! The record job as the script fakes it: notes made for each ask's id, or silence, as a switch the job never answers.

use super::{Call, Faked, Frame};
use crate::worker::{Heard, RecordFrames};
use kx_ipc::{RecordAsk, RecordNote};

/// What the record worker does with the next ask; with none queued it says nothing.
#[derive(Clone, Debug)]
pub enum RecordAct {
    /// These notes for the ask's id, in order.
    Say(Vec<fn(u32) -> RecordNote>),
    /// Nothing at all.
    Silent,
}

impl Faked for RecordFrames {
    type Act = RecordAct;

    fn usual() -> RecordAct {
        RecordAct::Silent
    }

    fn note(ask: RecordAsk<'static>) -> Option<(u32, Call)> {
        match ask {
            RecordAsk::Bye => None,
            ask => Some((ask.id(), Call::Record(ask))),
        }
    }

    fn answer(act: RecordAct, id: u32) -> Heard<RecordNote> {
        let first = Self::play(act, id).into_iter().next();
        first.map_or(Heard::Late, Heard::Frame)
    }

    fn play(act: RecordAct, id: u32) -> Vec<Frame<Self>> {
        match act {
            RecordAct::Say(notes) => notes.into_iter().map(|note| Ok(note(id))).collect(),
            RecordAct::Silent => Vec::new(),
        }
    }
}
