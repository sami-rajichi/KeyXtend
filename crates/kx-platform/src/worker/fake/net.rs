//! The speech-net job as the script fakes it: silent, as each test says the job's notes itself, in the order and at
//! the time it wants.

use super::{Call, Faked};
use crate::worker::{Heard, NetFrames};
use kx_ipc::{NetAsk, NetNote};

/// What the speech-net worker does with an ask: nothing on its own.
#[derive(Clone, Debug)]
pub struct NetAct;

impl Faked for NetFrames {
    type Act = NetAct;

    fn usual() -> NetAct {
        NetAct
    }

    fn note(ask: NetAsk<'static>) -> Option<(u32, Call)> {
        match ask {
            NetAsk::Bye => None,
            ask => Some((ask.id(), Call::Net(ask))),
        }
    }

    fn answer(_: NetAct, _: u32) -> Heard<NetNote> {
        Heard::Late
    }
}
