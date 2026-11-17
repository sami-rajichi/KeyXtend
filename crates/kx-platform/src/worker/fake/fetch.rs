//! The fetch job as the script fakes it: notes made for each ask's id, or silence.

use super::{Call, Faked, Frame};
use crate::worker::{FetchFrames, Heard};
use kx_ipc::{FetchAsk, FetchNote};

/// What the fetch worker does with the next ask; with none queued it says nothing, as a Pause or a Resume.
#[derive(Clone, Debug)]
pub enum FetchAct {
    /// These notes for the ask's id, in order.
    Say(Vec<fn(u32) -> FetchNote>),
    /// Nothing at all.
    Silent,
}

impl Faked for FetchFrames {
    type Act = FetchAct;

    fn usual() -> FetchAct {
        FetchAct::Silent
    }

    fn note(ask: FetchAsk<'static>) -> Option<(u32, Call)> {
        match ask {
            FetchAsk::Bye => None,
            ask => Some((ask.id(), Call::Fetch(ask))),
        }
    }

    fn answer(act: FetchAct, id: u32) -> Heard<FetchNote> {
        let first = Self::play(act, id).into_iter().next();
        first.map_or(Heard::Late, Heard::Frame)
    }

    fn play(act: FetchAct, id: u32) -> Vec<Frame<Self>> {
        match act {
            FetchAct::Say(notes) => notes.into_iter().map(|note| Ok(note(id))).collect(),
            FetchAct::Silent => Vec::new(),
        }
    }
}
