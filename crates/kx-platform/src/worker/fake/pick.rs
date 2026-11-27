//! The pick job as the script fakes it: one answer made for each dialog's id, silence, or a frame that cannot be read.

use super::{Call, Faked};
use crate::worker::{Heard, PickFrames};
use kx_ipc::{IpcError, PickAsk, PickReply};
use std::borrow::Cow;

/// What the pick worker does with the next dialog; with none queued it never answers, as a dialog left open.
#[derive(Clone, Debug)]
pub enum PickAct {
    /// This answer for the dialog's id.
    Answer(fn(u32) -> PickReply<'static>),
    /// The user picked this path.
    Path(String),
    /// Nothing at all.
    Silent,
    /// A frame that could not be read.
    Broken(IpcError),
}

impl Faked for PickFrames {
    type Act = PickAct;

    fn usual() -> PickAct {
        PickAct::Silent
    }

    fn note(ask: PickAsk<'static>) -> Option<(u32, Call)> {
        match ask {
            PickAsk::Bye => None,
            ask => Some((ask.id(), Call::Pick(ask))),
        }
    }

    fn answer(act: PickAct, id: u32) -> Heard<PickReply<'static>> {
        match act {
            PickAct::Answer(reply) => Heard::Frame(Ok(reply(id))),
            PickAct::Path(path) => Heard::Frame(Ok(PickReply::Picked {
                id,
                path: Cow::Owned(path),
            })),
            PickAct::Silent => Heard::Late,
            PickAct::Broken(e) => Heard::Frame(Err(e)),
        }
    }
}
